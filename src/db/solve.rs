use serenity::all::ChannelId;
use poise::macros::ChoiceParameter;
use strum::FromRepr;

#[derive(Debug, Clone)]
pub struct SolveRaw {
    pub id: i64,
    pub competition_id: i64,
    pub challenge_name: String,
    pub challenge_type: i64,
    pub flag: String,
    pub approved: bool,
}

impl From<Solve> for SolveRaw {
    fn from(value: Solve) -> Self {
        SolveRaw {
            id: value.id,
            competition_id: value.competition_id.get() as i64,
            challenge_name: value.challenge_name,
            challenge_type: value.challenge_type as i64,
            flag: value.flag,
            approved: value.approved,
        }
    }
}

#[repr(i64)]
#[derive(Debug, Clone, Copy, ChoiceParameter, FromRepr)]
pub enum ChallengeType {
    #[name = "rev"]
    Rev,
    #[name = "pwn"]
    Pwn,
    #[name = "web"]
    Web,
    #[name = "crypto"]
    Crypto,
    #[name = "misc"]
    Misc,
    #[name = "osint"]
    Osint,
    #[name = "forensics"]
    Forensics,
    #[name = "blockchain"]
    Blockchain,
}

#[derive(Debug, Clone)]
pub struct Solve {
    pub id: i64,
    pub competition_id: ChannelId,
    pub challenge_name: String,
    pub challenge_type: ChallengeType,
    pub flag: String,
    pub approved: bool,
}

impl From<SolveRaw> for Solve {
    fn from(value: SolveRaw) -> Self {
        Solve {
            id: value.id,
            competition_id: ChannelId::new(value.competition_id as u64),
            challenge_name: value.challenge_name,
            challenge_type: ChallengeType::from_repr(value.challenge_type)
                .expect("invalid challenge type returned from database"),
            flag: value.flag,
            approved: value.approved,
        }
    }
}