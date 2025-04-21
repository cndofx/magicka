use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponClass {
    Default = 0,
    ThrustFast,
    ThrustMedium,
    ThrustSlow,
    CrushFast,
    CrushMedium,
    CrushSlow,
    SlashFast,
    SlashMedium,
    SlashSlow,
    ThrowFast,
    ThrowMedium,
    ThrowSlow,
    Unarmed,
    Handgun,
    Rifle,
    MachineGun,
    Heavy,
    Staff,
    NewAnimationSet0,
    NewAnimationSet1,
    NewAnimationSet2,
    NewAnimationSet3,
    NewAnimationSet4,
    NewAnimationSet5,
    NewAnimationSet6,
    NewAnimationSet7,
}
