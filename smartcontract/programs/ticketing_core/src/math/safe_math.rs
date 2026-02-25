use crate::error::TicketingError;

pub trait SafeMath: Sized {
    fn safe_add(self, rhs: Self) -> Result<Self, TicketingError>;
    fn safe_sub(self, rhs: Self) -> Result<Self, TicketingError>;
    fn safe_mul(self, rhs: Self) -> Result<Self, TicketingError>;
}

impl SafeMath for u64 {
    fn safe_add(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_add(rhs).ok_or(TicketingError::MathOverflow)
    }

    fn safe_sub(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_sub(rhs).ok_or(TicketingError::MathOverflow)
    }

    fn safe_mul(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_mul(rhs).ok_or(TicketingError::MathOverflow)
    }
}

impl SafeMath for u32 {
    fn safe_add(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_add(rhs).ok_or(TicketingError::MathOverflow)
    }

    fn safe_sub(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_sub(rhs).ok_or(TicketingError::MathOverflow)
    }

    fn safe_mul(self, rhs: Self) -> Result<Self, TicketingError> {
        self.checked_mul(rhs).ok_or(TicketingError::MathOverflow)
    }
}

pub fn prorata_bps(total: u64, bps: u16) -> Result<u64, TicketingError> {
    let amount = (u128::from(total) * u128::from(bps))
        .checked_div(10_000u128)
        .ok_or(TicketingError::MathOverflow)?;
    amount.try_into().map_err(|_| TicketingError::MathOverflow)
}
