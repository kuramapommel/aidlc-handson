//! 業務の部品 Purchasing。
//!
//! 購入の可否を検証し、e チケットと予算の消費の取引を 1 つの決定として生成する（ADR-004）。
//! CourseCatalog・Eligibility・TrainingBudget を使う唯一の業務の部品で、データストアや
//! Web API には依存しない。チケット番号の乱数とチケットの取得は関数で受け取る。

pub mod decision;
pub mod model;
pub mod ticket_number;
pub mod tickets;

pub use decision::{consumption_entry_id, decide_purchase, first_rejection};
pub use model::{
    FormatError, PurchaseApproval, PurchaseDecision, PurchaseRequest, PurchaseSnapshot,
    RejectionReason, SessionAvailability, Ticket, TicketId, TicketIdentity, TicketNumber,
};
pub use ticket_number::{
    MAX_TICKET_NUMBER_ATTEMPTS, TICKET_NUMBER_ALPHABET, TicketNumberExhausted,
    generate_ticket_number, is_valid_ticket_number, ticket_number_candidate,
};
pub use tickets::{
    TicketView, find_own_ticket, has_purchased, is_number_taken, issued_counter, issued_counts,
    purchased_session_ids, session_availability, ticket_view,
};

#[cfg(test)]
mod tests;
