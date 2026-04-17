use anchor_lang::prelude::*;

/// Помилки програми Crafting
#[error_code]
pub enum CraftingError {
    /// Недійсний тип предмету (0-3)
    #[msg("Недійсний тип предмету для крафту (0-3)")]
    InvalidItemType,
    /// Недостатньо ресурсів для крафту
    #[msg("Недостатньо ресурсів для крафту предмету")]
    InsufficientResources,
    /// Неавторизований адміністратор
    #[msg("Доступ заборонено: не адміністратор")]
    UnauthorizedAdmin,
    /// Неправильний мінт ресурсу
    #[msg("Переданий мінт ресурсу не відповідає рецепту")]
    InvalidResourceMint,
}
