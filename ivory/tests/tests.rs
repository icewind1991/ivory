use maplit::hashmap;
use pretty_assertions::assert_eq;

use ivory::{ArrayKey, PhpVal};

#[test]
fn cast_into_php_val() {
    assert_eq!(PhpVal::Long(1), 1.into());
    assert_eq!(PhpVal::Double(1.1), (1.1).into());
    assert_eq!(PhpVal::String("foo".to_string()), "foo".to_string().into());
    assert_eq!(PhpVal::Bool(true), true.into());
    assert_eq!(PhpVal::Bool(false), false.into());
    assert_eq!(
        PhpVal::Array(vec![
            (ArrayKey::Int(0), PhpVal::Long(1)),
            (ArrayKey::Int(1), PhpVal::Long(2)),
            (ArrayKey::Int(2), PhpVal::Long(3))
        ]),
        vec![1, 2, 3].into()
    );
    assert_eq!(
        PhpVal::Array(vec![
            (ArrayKey::Int(0), PhpVal::Long(1)),
            (ArrayKey::Int(1), PhpVal::Double(2.1)),
            (ArrayKey::Int(2), PhpVal::String("3".to_string()))
        ]),
        vec![
            PhpVal::Long(1),
            PhpVal::Double(2.1),
            PhpVal::String("3".to_string())
        ]
        .into()
    );
    assert_eq!(
        PhpVal::Array(vec![
            (ArrayKey::Int(0), PhpVal::Long(1)),
            (ArrayKey::Int(3), PhpVal::Long(2)),
            (ArrayKey::Int(6), PhpVal::Long(3))
        ]),
        vec![(0u8, 1), (3, 2), (6, 3)].into()
    );
    assert_eq!(
        PhpVal::Array(vec![
            (ArrayKey::Int(0), PhpVal::Long(1)),
            (ArrayKey::Int(3), PhpVal::Long(2)),
            (ArrayKey::String("foo".to_string()), PhpVal::Long(3))
        ]),
        vec![
            (ArrayKey::Int(0), 1),
            (ArrayKey::Int(3), 2),
            (ArrayKey::String("foo".to_string()), 3)
        ]
        .into()
    );
    assert_eq!(
        PhpVal::Array(vec![
            (ArrayKey::Int(0), PhpVal::Long(1)),
            (ArrayKey::Int(3), PhpVal::Long(2)),
            (ArrayKey::Int(6), PhpVal::Long(3))
        ]),
        hashmap! {
            0u8 => 1,
            3 => 2,
            6 => 3
        }
        .into()
    );
}
