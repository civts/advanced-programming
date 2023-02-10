#[cfg(test)]
#[test]
fn test_format_float() {
    use crate::views::main_view::widgets::profit_chart::decimal_format_float;

    assert_eq!(decimal_format_float(1.0), "1.00");
    assert_eq!(decimal_format_float(1.49), "1.49");
    assert_eq!(decimal_format_float(1.51), "1.51");
    assert_eq!(decimal_format_float(13.0), "13.00");
    assert_eq!(decimal_format_float(13.23), "13.23");
    assert_eq!(decimal_format_float(13.63), "13.63");
    assert_eq!(decimal_format_float(130.6353), "130.64");
    assert_eq!(decimal_format_float(1200.342), "1.20");
    assert_eq!(decimal_format_float(1500.342), "1.50");
    assert_eq!(decimal_format_float(15000.342), "15.00");
    assert_eq!(decimal_format_float(15500.342), "15.50");
    assert_eq!(decimal_format_float(20434.122), "20.43");
    assert_eq!(decimal_format_float(50434.122), "50.43");
    assert_eq!(decimal_format_float(90434.122), "90.43");
    assert_eq!(decimal_format_float(590439.122), "590.44");
    assert_eq!(decimal_format_float(990434.122), "990.43");
    assert_eq!(decimal_format_float(1_001_000.122), "1.00");
    assert_eq!(decimal_format_float(1_006_000.122), "1.01");
    assert_eq!(decimal_format_float(1_060_000.122), "1.06");
    assert_eq!(decimal_format_float(1_600_000.122), "1.60");
    assert_eq!(decimal_format_float(960_000_000.122), "960.00");
    assert_eq!(decimal_format_float(1_000_000_000.0), "1.00");

    assert_eq!(decimal_format_float(-1.0), "-1.00");
    assert_eq!(decimal_format_float(-1.49), "-1.49");
    assert_eq!(decimal_format_float(-1.51), "-1.51");
    assert_eq!(decimal_format_float(-13.0), "-13.00");
    assert_eq!(decimal_format_float(-13.23), "-13.23");
    assert_eq!(decimal_format_float(-13.63), "-13.63");
    assert_eq!(decimal_format_float(-130.6353), "-130.64");
    assert_eq!(decimal_format_float(-1200.342), "-1.20");
    assert_eq!(decimal_format_float(-1500.342), "-1.50");
    assert_eq!(decimal_format_float(-15000.342), "-15.00");
    assert_eq!(decimal_format_float(-15500.342), "-15.50");
    assert_eq!(decimal_format_float(-20434.122), "-20.43");
    assert_eq!(decimal_format_float(-50434.122), "-50.43");
    assert_eq!(decimal_format_float(-90434.122), "-90.43");
    assert_eq!(decimal_format_float(-590439.122), "-590.44");
    assert_eq!(decimal_format_float(-990434.122), "-990.43");
    assert_eq!(decimal_format_float(-1_001_000.122), "-1.00");
    assert_eq!(decimal_format_float(-1_006_000.122), "-1.01");
    assert_eq!(decimal_format_float(-1_060_000.122), "-1.06");
    assert_eq!(decimal_format_float(-1_600_000.122), "-1.60");
    assert_eq!(decimal_format_float(-960_000_000.122), "-960.00");
    assert_eq!(decimal_format_float(-1_000_000_000.0), "-1.00");
}

#[test]
fn test_unit() {
    use crate::views::main_view::widgets::profit_chart::unit_of_measure;

    assert_eq!(unit_of_measure(1.0), ' ');
    assert_eq!(unit_of_measure(1.49), ' ');
    assert_eq!(unit_of_measure(1.51), ' ');
    assert_eq!(unit_of_measure(13.0), ' ');
    assert_eq!(unit_of_measure(13.23), ' ');
    assert_eq!(unit_of_measure(13.63), ' ');
    assert_eq!(unit_of_measure(130.6353), ' ');
    assert_eq!(unit_of_measure(1200.342), 'K');
    assert_eq!(unit_of_measure(1500.342), 'K');
    assert_eq!(unit_of_measure(15000.342), 'K');
    assert_eq!(unit_of_measure(15500.342), 'K');
    assert_eq!(unit_of_measure(20434.122), 'K');
    assert_eq!(unit_of_measure(50434.122), 'K');
    assert_eq!(unit_of_measure(90434.122), 'K');
    assert_eq!(unit_of_measure(590439.122), 'K');
    assert_eq!(unit_of_measure(990434.122), 'K');
    assert_eq!(unit_of_measure(1_001_000.122), 'M');
    assert_eq!(unit_of_measure(1_006_000.122), 'M');
    assert_eq!(unit_of_measure(1_060_000.122), 'M');
    assert_eq!(unit_of_measure(1_600_000.122), 'M');
    assert_eq!(unit_of_measure(960_000_000.122), 'M');
    assert_eq!(unit_of_measure(1_000_000_000.0), 'B');

    assert_eq!(unit_of_measure(-1.0), ' ');
    assert_eq!(unit_of_measure(-1.49), ' ');
    assert_eq!(unit_of_measure(-1.51), ' ');
    assert_eq!(unit_of_measure(-13.0), ' ');
    assert_eq!(unit_of_measure(-13.23), ' ');
    assert_eq!(unit_of_measure(-13.63), ' ');
    assert_eq!(unit_of_measure(-130.6353), ' ');
    assert_eq!(unit_of_measure(-1200.342), 'K');
    assert_eq!(unit_of_measure(-1500.342), 'K');
    assert_eq!(unit_of_measure(-15000.342), 'K');
    assert_eq!(unit_of_measure(-15500.342), 'K');
    assert_eq!(unit_of_measure(-20434.122), 'K');
    assert_eq!(unit_of_measure(-50434.122), 'K');
    assert_eq!(unit_of_measure(-90434.122), 'K');
    assert_eq!(unit_of_measure(-590439.122), 'K');
    assert_eq!(unit_of_measure(-990434.122), 'K');
    assert_eq!(unit_of_measure(-1_001_000.122), 'M');
    assert_eq!(unit_of_measure(-1_006_000.122), 'M');
    assert_eq!(unit_of_measure(-1_060_000.122), 'M');
    assert_eq!(unit_of_measure(-1_600_000.122), 'M');
    assert_eq!(unit_of_measure(-960_000_000.122), 'M');
    assert_eq!(unit_of_measure(-1_000_000_000.0), 'B');
}
