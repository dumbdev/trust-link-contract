#![no_std]
//! # TrustLink Escrow Contract
//!
//! A trustless escrow system for Stellar that enables secure peer-to-peer transactions
//! between buyers and sellers with optional dispute resolution.
//!
//! ## Overview
//!
//! This contract implements a state machine for escrow transactions where:
//! - Sellers create escrows specifying terms and a trusted resolver
//! - Buyers fund escrows by locking tokens in the contract
//! - Funds are released upon delivery confirmation or after a shipping window expires
//! - Disputes can be raised and resolved by a designated third-party resolver
//! - Protocol fees are collected on all successful settlements
//!
//! ## State Machine
//!
//! ```text
//! Pending -> Funded -> Completed
//!              |
//!              +-----> Disputed -> Completed/Refunded
//! ```

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

/// Maximum protocol fee in basis points (300 = 3%).
const MAX_FEE_BPS: u32 = 300;

/// Storage keys for persisting escrow data and the global escrow counter.
#[contracttype]
pub enum DataKey {
    /// Key for accessing a specific escrow by its unique ID.
    Escrow(u32),
    /// Key for the global counter tracking the total number of escrows created.
    EscrowCount,
    /// Key for the protocol fee collector address.
    FeeCollector,
}

/// Complete escrow record containing all transaction details and current state.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowData {
    /// Address of the seller who will receive funds upon successful completion.
    pub seller: Address,
    /// Address of the buyer who funds the escrow. None until the escrow is funded.
    pub buyer: Option<Address>,
    /// Address of the trusted third-party resolver who can mediate disputes.
    pub resolver: Address,
    /// Address of the token contract (SEP-41 compliant) used for the escrow.
    pub token: Address,
    /// Amount of tokens locked in the escrow.
    pub amount: i128,
    /// Protocol fee in basis points (100 = 1%).
    pub fee_bps: u32,
    /// Time window in seconds after funding during which auto-release is not allowed.
    pub shipping_window: u64,
    /// Ledger timestamp when the escrow was funded. Zero if not yet funded.
    pub funded_at: u64,
    /// Current lifecycle state of the escrow.
    pub state: EscrowState,
}

/// Lifecycle states of an escrow transaction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowState {
    /// Escrow created but not yet funded by a buyer.
    Pending,
    /// Escrow funded and awaiting delivery confirmation or dispute.
    Funded,
    /// Escrow successfully completed with funds released to the seller.
    Completed,
    /// Escrow in dispute, awaiting resolver decision.
    Disputed,
    /// Escrow refunded to the buyer after dispute resolution.
    Refunded,
}

/// Protocol fee configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeConfig {
    /// Address that receives protocol fees.
    pub collector: Address,
    /// Maximum allowed fee in basis points.
    pub max_fee_bps: u32,
}

/// TrustLink escrow contract implementation.
#[contract]
pub struct Escrow;

/// Calculates the protocol fee, transfers it to the fee collector, and sends
/// the remainder to the designated recipient. This is the single source of
/// truth for all outbound escrow disbursements.
fn deduct_and_transfer(env: &Env, token_addr: &Address, recipient: &Address, amount: i128, fee_bps: u32) {
    let fee = amount
        .checked_mul(fee_bps as i128)
        .expect("fee overflow")
        / 10_000i128;
    let net = amount.checked_sub(fee).expect("fee underflow");

    let token_client = token::Client::new(env, token_addr);

    if fee > 0 {
        let collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::FeeCollector)
            .expect("fee collector not set");
        token_client.transfer(&env.current_contract_address(), &collector, &fee);
    }

    token_client.transfer(&env.current_contract_address(), recipient, &net);
}

#[contractimpl]
#[allow(deprecated)]
impl Escrow {
    /// Initializes the contract with a protocol fee collector address.
    ///
    /// This function must be called once before any escrow settlements can occur.
    /// It sets the address that will receive protocol fees from all escrow completions.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `fee_collector` - The address that will receive all protocol fees.
    ///
    /// # Returns
    ///
    /// Returns `()` on success.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The contract has already been initialized (fee collector is already set).
    ///
    /// # Auth
    ///
    /// No authorization required for the initial setup. However, this function can
    /// only be called once, preventing unauthorized changes to the fee collector.
    pub fn initialize(env: Env, fee_collector: Address) {
        if env
            .storage()
            .instance()
            .has(&DataKey::FeeCollector)
        {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&DataKey::FeeCollector, &fee_collector);
    }

    /// Creates a new escrow transaction in the Pending state.
    ///
    /// This function initializes an escrow with the specified parameters and assigns it
    /// a unique sequential ID. The escrow remains in the Pending state until a buyer
    /// funds it via `fund_escrow`.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `seller` - The address that will receive funds upon successful completion.
    /// * `resolver` - The address authorized to resolve disputes if they arise.
    /// * `token` - The address of the SEP-41 token contract to be used for payment.
    /// * `amount` - The quantity of tokens to be locked in escrow (must be positive).
    /// * `fee_bps` - Protocol fee in basis points (100 = 1%, max 300 = 3%).
    /// * `shipping_window` - Duration in seconds after funding before auto-release is permitted.
    ///
    /// # Returns
    ///
    /// Returns the unique escrow ID (u32) assigned to this escrow. IDs start at 1 and
    /// increment sequentially.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The seller address fails authentication (does not sign the transaction).
    /// - The `fee_bps` exceeds the maximum allowed fee (300 basis points).
    /// - Storage operations fail (extremely rare in normal operation).
    ///
    /// # Auth
    ///
    /// Requires authorization from the `seller` address. The seller must sign this
    /// transaction to prove they are creating the escrow.
    pub fn create_escrow(
        env: Env,
        seller: Address,
        resolver: Address,
        token: Address,
        amount: i128,
        fee_bps: u32,
        shipping_window: u64,
    ) -> u32 {
        seller.require_auth();
        assert!(fee_bps <= MAX_FEE_BPS, "fee exceeds maximum");

        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0);
        count += 1;

        let escrow = EscrowData {
            seller,
            buyer: None,
            resolver,
            token,
            amount,
            fee_bps,
            shipping_window,
            funded_at: 0,
            state: EscrowState::Pending,
        };

        env.storage()
            .instance()
            .set(&DataKey::Escrow(count), &escrow);
        env.storage()
            .instance()
            .set(&DataKey::EscrowCount, &count);

        env.events().publish(("create_escrow",), count);
        count
    }

    /// Funds an existing escrow by locking tokens from the buyer into the contract.
    ///
    /// This function transitions an escrow from Pending to Funded state. The buyer's
    /// tokens are transferred to the contract address and held until the escrow is
    /// completed, disputed, or refunded. The funding timestamp is recorded to enable
    /// time-based auto-release.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the escrow to fund.
    /// * `buyer` - The address providing the funds and becoming the buyer in this escrow.
    ///
    /// # Returns
    ///
    /// Returns `()` on success. The escrow state is updated to Funded and the buyer
    /// address is recorded.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The escrow with the given `escrow_id` does not exist.
    /// - The escrow is not in the Pending state (already funded, completed, etc.).
    /// - The buyer address fails authentication (does not sign the transaction).
    /// - The token transfer fails (insufficient balance, token contract error, etc.).
    ///
    /// # Auth
    ///
    /// Requires authorization from the `buyer` address. The buyer must sign this
    /// transaction and must have sufficient token balance and allowance for the transfer.
    pub fn fund_escrow(env: Env, escrow_id: u32, buyer: Address) {
        buyer.require_auth();

        let mut escrow: EscrowData = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(escrow.state == EscrowState::Pending, "escrow not pending");

        escrow.buyer = Some(buyer.clone());
        escrow.state = EscrowState::Funded;
        escrow.funded_at = env.ledger().timestamp();

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(&buyer, env.current_contract_address(), &escrow.amount);

        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &escrow);
        env.events().publish(("fund_escrow",), escrow_id);
    }

    /// Confirms successful delivery and releases escrowed funds to the seller.
    ///
    /// This function allows the buyer to confirm that goods or services have been
    /// delivered satisfactorily. Upon confirmation, the locked tokens are immediately
    /// transferred from the contract to the seller (minus protocol fee), and the escrow
    /// transitions to the Completed state.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the escrow to complete.
    ///
    /// # Returns
    ///
    /// Returns `()` on success. The tokens are transferred to the seller (after fee
    /// deduction) and the escrow state is updated to Completed.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The escrow with the given `escrow_id` does not exist.
    /// - The escrow is not in the Funded state (not yet funded, already completed, etc.).
    /// - The buyer address fails authentication (does not sign the transaction).
    /// - The escrow has no buyer recorded (should never happen if state is Funded).
    /// - The token transfer fails (contract balance insufficient, token contract error).
    /// - The fee collector has not been initialized.
    ///
    /// # Auth
    ///
    /// Requires authorization from the buyer address recorded in the escrow. Only the
    /// buyer who funded the escrow can confirm delivery.
    pub fn confirm_delivery(env: Env, escrow_id: u32) {
        let escrow: EscrowData = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(escrow.state == EscrowState::Funded, "escrow not funded");

        let buyer = escrow.buyer.clone().expect("escrow has no buyer");
        buyer.require_auth();

        deduct_and_transfer(&env, &escrow.token, &escrow.seller, escrow.amount, escrow.fee_bps);

        let mut updated = escrow;
        updated.state = EscrowState::Completed;

        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &updated);
        env.events().publish(("confirm_delivery",), escrow_id);
    }

    /// Raises a dispute on a funded escrow with cryptographic evidence.
    ///
    /// This function allows the buyer to challenge the transaction by providing a
    /// 32-byte hash of off-chain evidence (such as a SHA-256 digest of photos,
    /// messages, or documents). The escrow transitions to the Disputed state and
    /// funds remain locked until the resolver makes a decision via `resolve_dispute`.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the escrow to dispute.
    /// * `evidence_hash` - A 32-byte cryptographic hash of the dispute evidence.
    ///
    /// # Returns
    ///
    /// Returns `()` on success. The escrow state is updated to Disputed and the
    /// evidence hash is emitted in the event log.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The `evidence_hash` is not exactly 32 bytes in length.
    /// - The escrow with the given `escrow_id` does not exist.
    /// - The escrow is not in the Funded state (cannot dispute pending or completed escrows).
    /// - The buyer address fails authentication (does not sign the transaction).
    /// - The escrow has no buyer recorded (should never happen if state is Funded).
    ///
    /// # Auth
    ///
    /// Requires authorization from the buyer address recorded in the escrow. Only the
    /// buyer who funded the escrow can raise a dispute.
    pub fn raise_dispute(env: Env, escrow_id: u32, evidence_hash: soroban_sdk::Bytes) {
        assert!(
            evidence_hash.len() == 32,
            "evidence_hash must be exactly 32 bytes"
        );

        let escrow: EscrowData = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(escrow.state == EscrowState::Funded, "escrow not funded");

        let buyer = escrow.buyer.clone().expect("escrow has no buyer");
        buyer.require_auth();

        let mut updated = escrow;
        updated.state = EscrowState::Disputed;

        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &updated);
        env.events()
            .publish(("raise_dispute",), (escrow_id, evidence_hash));
    }

    /// Resolves a disputed escrow by releasing funds to either the seller or buyer.
    ///
    /// This function allows the designated resolver to make a final decision on a
    /// disputed escrow. Based on the `release_to_seller` parameter, funds are either
    /// transferred to the seller (if the dispute is resolved in their favor) or
    /// refunded to the buyer. Protocol fees are deducted in both cases. The escrow
    /// transitions to either Completed or Refunded state accordingly.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the disputed escrow to resolve.
    /// * `release_to_seller` - If true, funds go to the seller; if false, refunded to buyer.
    ///
    /// # Returns
    ///
    /// Returns `()` on success. The tokens are transferred to the appropriate party
    /// (after fee deduction) and the escrow state is updated to Completed or Refunded.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The escrow with the given `escrow_id` does not exist.
    /// - The escrow is not in the Disputed state (cannot resolve non-disputed escrows).
    /// - The resolver address fails authentication (does not sign the transaction).
    /// - The escrow has no buyer recorded (should never happen if state is Disputed).
    /// - The token transfer fails (contract balance insufficient, token contract error).
    /// - The fee collector has not been initialized.
    ///
    /// # Auth
    ///
    /// Requires authorization from the resolver address specified when the escrow was
    /// created. Only the designated resolver can make dispute resolution decisions.
    pub fn resolve_dispute(env: Env, escrow_id: u32, release_to_seller: bool) {
        let escrow: EscrowData = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(escrow.state == EscrowState::Disputed, "escrow not disputed");

        escrow.resolver.require_auth();

        let recipient = if release_to_seller {
            escrow.seller.clone()
        } else {
            escrow.buyer.clone().expect("escrow has no buyer")
        };

        deduct_and_transfer(&env, &escrow.token, &recipient, escrow.amount, escrow.fee_bps);

        let mut updated = escrow;
        updated.state = if release_to_seller {
            EscrowState::Completed
        } else {
            EscrowState::Refunded
        };

        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &updated);
        env.events()
            .publish(("resolve_dispute",), (escrow_id, release_to_seller));
    }

    /// Automatically releases escrowed funds to the seller after the shipping window expires.
    ///
    /// This function provides a permissionless mechanism to release funds to the seller
    /// when the buyer has not confirmed delivery or raised a dispute within the
    /// specified shipping window. Anyone can call this function once the time condition
    /// is met, preventing funds from being locked indefinitely. Protocol fees are
    /// deducted from the transfer.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the escrow to auto-release.
    ///
    /// # Returns
    ///
    /// Returns `()` on success. The tokens are transferred to the seller (after fee
    /// deduction) and the escrow state is updated to Completed.
    ///
    /// # Errors
    ///
    /// This function panics if:
    /// - The escrow with the given `escrow_id` does not exist.
    /// - The escrow is not in the Funded state (cannot auto-release pending or completed escrows).
    /// - The shipping window has not yet elapsed (current timestamp < funded_at + shipping_window).
    /// - The token transfer fails (contract balance insufficient, token contract error).
    /// - The fee collector has not been initialized.
    ///
    /// # Auth
    ///
    /// No authorization required. This function is intentionally permissionless and can
    /// be called by anyone once the time-based condition is satisfied. This ensures
    /// sellers can always receive payment even if the buyer becomes unresponsive.
    pub fn auto_release(env: Env, escrow_id: u32) {
        let escrow: EscrowData = env
            .storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found");

        assert!(escrow.state == EscrowState::Funded, "escrow not funded");
        assert!(
            env.ledger().timestamp() >= escrow.funded_at + escrow.shipping_window,
            "shipping window not elapsed"
        );

        deduct_and_transfer(&env, &escrow.token, &escrow.seller, escrow.amount, escrow.fee_bps);

        let mut updated = escrow;
        updated.state = EscrowState::Completed;

        env.storage()
            .instance()
            .set(&DataKey::Escrow(escrow_id), &updated);
        env.events().publish(("auto_release",), escrow_id);
    }

    /// Retrieves the complete data for a specific escrow.
    ///
    /// This is a read-only view function that returns all stored information about
    /// an escrow, including its current state, participant addresses, token details,
    /// amounts, and timestamps.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    /// * `escrow_id` - The unique identifier of the escrow to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the complete [`EscrowData`] struct containing all escrow information.
    ///
    /// # Errors
    ///
    /// This function panics if the escrow with the given `escrow_id` does not exist.
    ///
    /// # Auth
    ///
    /// No authorization required. This is a public read-only function that can be
    /// called by anyone to inspect escrow state.
    pub fn get_escrow(env: Env, escrow_id: u32) -> EscrowData {
        env.storage()
            .instance()
            .get(&DataKey::Escrow(escrow_id))
            .expect("escrow not found")
    }

    /// Retrieves the current protocol fee configuration.
    ///
    /// This is a read-only view function that returns the fee collector address
    /// and the maximum allowed fee in basis points.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment providing access to ledger state and storage.
    ///
    /// # Returns
    ///
    /// Returns a [`FeeConfig`] struct containing the collector address and maximum fee.
    ///
    /// # Errors
    ///
    /// This function panics if the fee collector has not been initialized.
    ///
    /// # Auth
    ///
    /// No authorization required. This is a public read-only function that can be
    /// called by anyone to inspect the fee configuration.
    pub fn get_fee_config(env: Env) -> FeeConfig {
        let collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::FeeCollector)
            .expect("fee collector not set");

        FeeConfig {
            collector,
            max_fee_bps: MAX_FEE_BPS,
        }
    }
}

mod test;
