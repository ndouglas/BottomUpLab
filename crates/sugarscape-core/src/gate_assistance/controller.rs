use super::types::*;

/// Choose exactly one primitive from the active Agent's own schedule and state.
pub fn choose(view: &View) -> Result<Primitive, Error> {
    view.validate()?;
    if view.clock.phase != Phase::BeforeAction || view.clock.role != Some(view.role) {
        return Err(history_error("choose requires an active BeforeAction View"));
    }
    own_primitive(
        view.role,
        view.word.token(view.clock.round)?,
        view.own_cell,
        view.own_cargo,
    )
}

pub(super) fn own_primitive(
    role: Role,
    token: Token,
    cell: Cell,
    cargo: bool,
) -> Result<Primitive, Error> {
    match (role, token) {
        (Role::Holder, Token::Holder(token)) if cell != Cell::C && !cargo => Ok(match token {
            HolderToken::Wait => Primitive::Wait,
            HolderToken::Travel => Primitive::Move(Cell::P),
            HolderToken::Hold => Primitive::OperatePlate,
        }),
        (Role::Retriever, Token::Retriever(RetrieverToken::Wait)) if cell != Cell::P => {
            Ok(Primitive::Wait)
        }
        (Role::Retriever, Token::Retriever(RetrieverToken::Advance)) => match (cell, cargo) {
            (Cell::O, false) => Ok(Primitive::Move(Cell::C)),
            (Cell::C, false) => Ok(Primitive::Pickup),
            (Cell::C, true) => Ok(Primitive::Move(Cell::O)),
            (Cell::O, true) => Ok(Primitive::Deliver),
            _ => Err(history_error("Retriever own phase")),
        },
        _ => Err(history_error("role/word/own state mismatch")),
    }
}
