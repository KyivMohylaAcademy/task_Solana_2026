use anchor_lang::prelude::*;

#[error_code]
pub enum CraftingError {
    #[msg("Невірний тип предмета. Допустимо: 0–3.")]
    InvalidItemType,
    #[msg("Недостатньо ресурсів для крафту.")]
    InsufficientResources,
}
