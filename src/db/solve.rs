use serenity::all::{ChannelId, MessageId};
use poise::macros::ChoiceParameter;
use strum::FromRepr;

#[derive(Debug, Clone)]
pub struct SolveRaw {
    pub id: i64,
    pub competition_id: i64,
    pub approval_message_id: i64,
    pub challenge_name: String,
    pub challenge_type: i64,
    pub flag: String,
    pub approved: i64,
}

impl From<Solve> for SolveRaw {
    fn from(value: Solve) -> Self {
        SolveRaw {
            id: value.id,
            competition_id: value.competition_id.get() as i64,
            approval_message_id: value.approval_message_id.get() as i64,
            challenge_name: value.challenge_name,
            challenge_type: value.challenge_type as i64,
            flag: value.flag,
            approved: value.approved as i64,
        }
    }
}

#[repr(i64)]
#[derive(Debug, Clone, Copy, ChoiceParameter, FromRepr, strum::Display)]
pub enum ChallengeType {
    #[name = "rev"]
    #[strum(to_string = "rev")]
    Rev,
    #[name = "pwn"]
    #[strum(to_string = "pwn")]
    Pwn,
    #[name = "web"]
    #[strum(to_string = "web")]
    Web,
    #[name = "crypto"]
    #[strum(to_string = "crypto")]
    Crypto,
    #[name = "misc"]
    #[strum(to_string = "misc")]
    Misc,
    #[name = "osint"]
    #[strum(to_string = "osint")]
    Osint,
    #[name = "forensics"]
    #[strum(to_string = "forensics")]
    Forensics,
    #[name = "blockchain"]
    #[strum(to_string = "blockchain")]
    Blockchain,
}

#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr, strum::Display)]
pub enum ApprovalStatus {
    Pending = 0,
    Approved = 1,
    Declined = 2,
}

#[derive(Debug, Clone)]
pub struct Solve {
    pub id: i64,
    pub competition_id: ChannelId,
    pub approval_message_id: MessageId,
    pub challenge_name: String,
    pub challenge_type: ChallengeType,
    pub flag: String,
    pub approved: ApprovalStatus,
}

impl From<SolveRaw> for Solve {
    fn from(value: SolveRaw) -> Self {
        Solve {
            id: value.id,
            competition_id: ChannelId::new(value.competition_id as u64),
            approval_message_id: MessageId::new(value.approval_message_id as u64),
            challenge_name: value.challenge_name,
            challenge_type: ChallengeType::from_repr(value.challenge_type)
                .expect("invalid challenge type returned from database"),
            flag: value.flag,
            approved: ApprovalStatus::from_repr(value.approved)
                .expect("invalid approval status returned from database"),
        }
    }
}