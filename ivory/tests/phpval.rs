#[link(libphp)]
use maplit::hashmap;
use pretty_assertions::assert_eq;

use ivory::zend::ZVal;
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
            (ArrayKey::String("asd".to_string()), PhpVal::Long(1)),
            (ArrayKey::String("foo".to_string()), PhpVal::Long(2)),
            (ArrayKey::String("bar".to_string()), PhpVal::Long(3))
        ]),
        vec![
            ("asd".to_string(), 1),
            ("foo".to_string(), 2),
            ("bar".to_string(), 3)
        ]
        .into()
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

#[test]
fn cast_into_php_val_round_trip() {
    let values: Vec<PhpVal> = vec![
        1.into(),
        (0.2).into(),
        true.into(),
        false.into(),
        "foo".to_string().into(),
    ];

    for original in values {
        let first_cast: ZVal = original.clone().into();
        let cast_back: PhpVal = first_cast.as_php_val();
        assert_eq!(original, cast_back);

        // the first round trip should cast back into the same round trip
        let second_cast: ZVal = cast_back.into();
        let cast_back: PhpVal = second_cast.as_php_val();
        assert_eq!(original, cast_back);
    }
}
