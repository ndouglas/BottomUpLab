use super::types::*;
/// Choice consumes only the active Agent's own word and local interface.
pub fn choose(view: &View) -> Result<Intent, Error> {
    view.validate()?;
    if view.clock.phase != Phase::BeforeAction || view.clock.role != Some(view.role) {
        return Err(history_error("choose requires own BeforeAction"));
    }
    own_intent(
        view.role,
        view.word.token(view.clock.round)?,
        view.own_cell,
        view.own_cargo,
    )
}
pub(super) fn own_intent(
    role: Role,
    token: Token,
    cell: Cell,
    cargo: bool,
) -> Result<Intent, Error> {
    match (role, token) {
        (Role::Giver, Token::Giver(t)) if cell != Cell::C && !cargo => Ok(match t {
            GiverToken::Wait => Intent::Wait,
            GiverToken::Approach => Intent::Move(Cell::O),
            GiverToken::Pass => Intent::Pass,
        }),
        (Role::Receiver, Token::Receiver(ReceiverToken::Wait)) if cell != Cell::P => {
            Ok(Intent::Wait)
        }
        (Role::Receiver, Token::Receiver(ReceiverToken::Advance)) => match (cell, cargo) {
            (Cell::O, false) => Ok(Intent::Move(Cell::C)),
            (Cell::C, false) => Ok(Intent::Pickup),
            (Cell::C, true) => Ok(Intent::Move(Cell::O)),
            (Cell::O, true) => Ok(Intent::Deliver),
            _ => Err(history_error("Receiver position")),
        },
        _ => Err(history_error("role, word or own state mismatch")),
    }
}
