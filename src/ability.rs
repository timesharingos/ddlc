use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Ability {
    Strength(i32),
    Dexterity(i32),
    Stamina(i32),
    Endurement(i32),
    Luck(i32),
    Intelligence(i32),
}

impl Ability {
    pub fn typename(&self) -> String {
        String::from(match &self {
            Self::Strength(_) => "Strength",
            Self::Dexterity(_) => "Dexterity",
            Self::Stamina(_) => "Stamina",
            Self::Endurement(_) => "Endurement",
            Self::Luck(_) => "Luck",
            Self::Intelligence(_) => "Intelligence",
        })
    }
    pub fn value(&self) -> i32 {
        match &self {
            Self::Strength(value) => *value,
            Self::Dexterity(value) => *value,
            Self::Stamina(value) => *value,
            Self::Endurement(value) => *value,
            Self::Luck(value) => *value,
            Self::Intelligence(value) => *value,
        }
    }
}

pub struct AbilityModifier {
    ability: Ability,
    modifier_positive: [i32; 2],
    modifier_negative: [i32; 2],
    current: Option<i32>,
}

impl From<Ability> for AbilityModifier {
    fn from(ability: Ability) -> Self {
        Self {
            ability,
            modifier_positive: [0; 2],
            modifier_negative: [0; 2],
            current: Option::None,
        }
    }
}

impl AbilityModifier {
    pub fn apply_positive(&mut self, modifier: i32) {
        if modifier >= self.modifier_positive[0] {
            self.modifier_positive = [modifier, self.modifier_positive[0]];
            self.current = None;
        } else if modifier >= self.modifier_positive[1] {
            self.modifier_positive = [self.modifier_positive[0], modifier];
            self.current = None;
        }
    }

    pub fn apply_negative(&mut self, modifier: i32) {
        let mut modifier = modifier;
        if modifier > 0 {
            modifier = (-1) * modifier;
        }
        if modifier <= self.modifier_negative[0] {
            self.modifier_negative = [modifier, self.modifier_negative[0]];
            self.current = None;
        } else if modifier <= self.modifier_negative[1] {
            self.modifier_negative = [self.modifier_negative[0], modifier];
            self.current = None;
        }
    }

    pub fn value(&mut self) -> i32 {
        if let Some(val) = self.current {
            return val;
        }
        let val = self.ability.value()
            + self.modifier_positive.iter().sum::<i32>()
            + self.modifier_negative.iter().sum::<i32>();
        self.current = Some(val);
        val
    }
}

#[derive(Debug)]
pub enum AbilityModelType {
    Equal,
    WeigthedOnPrior,
    Single,
}

pub struct AbilityModel {
    model_type: AbilityModelType,
    ability1: Rc<RefCell<AbilityModifier>>,
    ability2: Option<Rc<RefCell<AbilityModifier>>>,
}

impl AbilityModel {
    pub fn new(
        model_type: AbilityModelType,
        ability1: Rc<RefCell<AbilityModifier>>,
        ability2: Option<Rc<RefCell<AbilityModifier>>>,
    ) -> Result<Self, AbilityModelType> {
        match model_type {
            AbilityModelType::Single => Ok(Self {
                model_type,
                ability1,
                ability2,
            }),
            _ => match ability2 {
                None => Err(model_type),
                Some(_) => Ok(Self {
                    model_type,
                    ability1,
                    ability2,
                }),
            },
        }
    }

    pub fn value(&self) -> i32 {
        match &self.model_type {
            &AbilityModelType::Single => self.ability1.borrow_mut().value(),
            &AbilityModelType::Equal => {
                (self.ability1.borrow_mut().value()
                    + self
                        .ability2
                        .as_ref()
                        .expect("ability2 should exist")
                        .borrow_mut()
                        .value())
                    / 2
            }
            &AbilityModelType::WeigthedOnPrior => {
                (self.ability1.borrow_mut().value() * 2
                    + self
                        .ability2
                        .as_ref()
                        .expect("ability2 should exist")
                        .borrow_mut()
                        .value())
                    / 3
            }
        }
    }
}

impl Display for Ability {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "\t{}: {}\n", self.typename(), self.value())
    }
}

impl Display for AbilityModifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "[{}, modifiers: {},{},{},{}] -> {}",
            self.ability,
            self.modifier_positive[0],
            self.modifier_positive[1],
            self.modifier_negative[0],
            self.modifier_negative[1],
            self.current
                .map_or(String::from("<Lazy>"), |val| format!("{}", val))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Ability::Luck(105).to_string(), "\tLuck: 105\n");
    }

    #[test]
    fn biased_ability() {
        let ability_stamina: Rc<RefCell<AbilityModifier>> =
            Rc::new(RefCell::new(Ability::Stamina(100).into()));
        let ability_endurement: Rc<RefCell<AbilityModifier>> =
            Rc::new(RefCell::new(Ability::Endurement(160).into()));
        let defense = AbilityModel::new(
            AbilityModelType::WeigthedOnPrior,
            ability_stamina.clone(),
            Some(ability_endurement.clone()),
        )
        .expect("it should succeed.");
        assert_eq!(defense.value(), 120);
        ability_stamina.borrow_mut().apply_positive(60);
        assert_eq!(defense.value(), 160);
        ability_endurement.borrow_mut().apply_negative(60);
        assert_eq!(defense.value(), 140);
    }

    #[test]
    fn multiple_modifier() {
        let ability_intelligence: Rc<RefCell<AbilityModifier>> =
            Rc::new(RefCell::new(Ability::Intelligence(100).into()));
        let buff = AbilityModel::new(AbilityModelType::Single, ability_intelligence.clone(), None)
            .expect("it should succeed");
        assert_eq!(buff.value(), 100);
        ability_intelligence.borrow_mut().apply_positive(30);
        ability_intelligence.borrow_mut().apply_positive(50);
        ability_intelligence.borrow_mut().apply_positive(40);
        assert_eq!(buff.value(), 190);
        ability_intelligence.borrow_mut().apply_negative(-40);
        ability_intelligence.borrow_mut().apply_negative(-50);
        ability_intelligence.borrow_mut().apply_negative(-60);
        assert_eq!(buff.value(), 80);
    }
}
