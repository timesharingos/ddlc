use crate::ability::AbilityModel;
use std::fmt::{self, Display, Formatter};

pub struct SkillEffect {
    name: String,
    min: f32,
    max: f32,
    border: i32,
}

pub struct Skill {
    name: String,
    effects: Vec<SkillEffect>,
}

impl SkillEffect {
    pub fn new(name: &str, min: f32, max: f32, border: i32) -> Self {
        Self {
            name: name.to_owned(),
            min,
            max,
            border,
        }
    }

    pub fn new_damage(name: &str, min: f32, border: i32) -> Self {
        Self::new(name, min, min * 5.0, border)
    }

    pub fn new_recover(name: &str, min: f32, border: i32) -> Self {
        Self::new(name, min, min * 3.0, border)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn border(&self) -> i32 {
        self.border
    }

    pub fn reset_damage(&mut self, min: f32, max: f32) {
        self.min = min;
        self.max = max;
    }

    fn cause_damage(&self, sd: i32, border_factor: f32) -> f32 {
        let min_threshold = (-1.0) * self.border as f32 * border_factor;
        if sd >= self.border {
            return self.max;
        } else if sd as f32 <= min_threshold {
            return 1.0;
        } else if sd >= 0 {
            return self.min + (self.max - self.min) / self.border as f32 * sd as f32;
        } else {
            return 1.0 + (self.min - 1.0) / min_threshold * sd as f32;
        }
    }
    fn cause_effect(&self, sd: i32) -> f32 {
        if sd >= self.border {
            return self.max;
        } else if sd <= 0 {
            return self.min;
        } else {
            return self.min + (self.max - self.min) / self.border as f32 * sd as f32;
        }
    }

    pub fn damage_from_enemy(
        &self,
        attacker_model: &AbilityModel,
        defender_model: &AbilityModel,
    ) -> f32 {
        self.cause_damage(attacker_model.value() - defender_model.value(), 0.5)
    }
    pub fn damage_to_enemy(
        &self,
        attacker_model: &AbilityModel,
        defender_model: &AbilityModel,
    ) -> f32 {
        self.cause_damage(attacker_model.value() - defender_model.value(), 1.0)
    }
    pub fn effect_oneside(&self, attacker_model: &AbilityModel) -> f32 {
        self.cause_effect(attacker_model.value())
    }
    pub fn effect_attack(
        &self,
        attacker_model: &AbilityModel,
        defender_model: &AbilityModel,
    ) -> f32 {
        self.cause_effect(attacker_model.value() - defender_model.value())
    }
}

impl Skill {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            effects: vec![],
        }
    }

    pub fn add_effect(&mut self, effect: SkillEffect) {
        self.effects.push(effect);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for SkillEffect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}-{}/{}",
            self.name, self.min, self.max, self.border
        )
    }
}

impl Display for Skill {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] Effects:[{}]",
            self.name,
            self.effects
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::*;

    #[test]
    fn cause_damage() {
        let attacker_ability: AbilityModifierHelper =
            AbilityModifier::from(Ability::Intelligence(300)).into();
        let defender_ability: AbilityModifierHelper =
            AbilityModifier::from(Ability::Stamina(100)).into();
        let attacker_model =
            AbilityModel::new(AbilityModelType::Single, attacker_ability.get_cell(), None)
                .expect("it should succeed");
        let defender_model =
            AbilityModel::new(AbilityModelType::Equal, defender_ability.get_cell(), None)
                .expect("it should succeed");
        let skill_effect = SkillEffect::new_damage("damage1", 2000.0, 150);
        // too low (100 to 300)
        assert_eq!(
            skill_effect.damage_from_enemy(&defender_model, &attacker_model),
            1.0
        );
        assert_eq!(
            skill_effect.damage_to_enemy(&defender_model, &attacker_model),
            1.0
        );
        //weak from
        defender_ability.get_mut().apply_positive(100);
    }
}
