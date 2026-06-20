#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteValue(i16);

impl VoteValue {
    pub fn new(value: i16) -> Option<Self> {
        match value {
            -1 | 1 => Some(Self(value)),
            _ => None,
        }
    }

    pub fn value(self) -> i16 {
        self.0
    }
}

pub fn vote_delta(new_vote: VoteValue, previous_vote: Option<VoteValue>) -> i32 {
    i32::from(new_vote.value() - previous_vote.map(VoteValue::value).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_value_allows_only_upvote_or_downvote() {
        assert_eq!(VoteValue::new(1).map(VoteValue::value), Some(1));
        assert_eq!(VoteValue::new(-1).map(VoteValue::value), Some(-1));
        assert!(VoteValue::new(0).is_none());
    }

    #[test]
    fn vote_delta_handles_new_and_replaced_votes() {
        assert_eq!(vote_delta(VoteValue::new(1).unwrap(), None), 1);
        assert_eq!(
            vote_delta(
                VoteValue::new(1).unwrap(),
                Some(VoteValue::new(-1).unwrap())
            ),
            2
        );
    }
}
