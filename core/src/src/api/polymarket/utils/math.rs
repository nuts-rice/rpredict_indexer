pub trait ClobPrecision {
    fn round_normal(self, decimals: u32) -> Self;

    fn round_down(self, decimals: u32) -> Self;

    fn round_up(self, decimals: u32) -> Self;

    fn decimal_places(self) -> u32;
}

impl ClobPrecision for f64 {
    fn round_normal(self, decimals: u32) -> Self {
        if self.decimal_places() <= decimals {
            return self;
        }
        let factor = 10_f64.powi(decimals as i32);
        (self * factor).round() / factor
    }

    fn round_down(self, decimals: u32) -> Self {
        if self.decimal_places() <= decimals {
            return self;
        }
        let factor = 10_f64.powi(decimals as i32);
        (self * factor).floor() / factor
    }

    fn round_up(self, decimals: u32) -> Self {
        if self.decimal_places() <= decimals {
            return self;
        }
        let factor = 10_f64.powi(decimals as i32);
        (self * factor).ceil() / factor
    }

    fn decimal_places(self) -> u32 {
        if self.fract() == 0.0 {
            return 0;
        }

        let num_str = self.to_string();
        if let Some(pos) = num_str.find('.') {
            return (num_str.len() - pos - 1) as u32;
        }

        0
    }
}
pub fn adjust_amount(mut amount: f64, allowed_decimals: u32) -> f64 {
    if amount.decimal_places() > allowed_decimals {
        amount = amount.round_up(allowed_decimals + 4);
        if amount.decimal_places() > allowed_decimals {
            amount = amount.round_down(allowed_decimals);
        }
    }
    amount
}
