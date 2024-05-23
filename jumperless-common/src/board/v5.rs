//  _____   ____    _   _  ____ _______   ______ _____ _____ _______
// |  __ \ / __ \  | \ | |/ __ \__   __| |  ____|  __ \_   _|__   __|
// | |  | | |  | | |  \| | |  | | | |    | |__  | |  | || |    | |
// | |  | | |  | | | . ` | |  | | | |    |  __| | |  | || |    | |
// | |__| | |__| | | |\  | |__| | | |    | |____| |__| || |_   | |
// |_____/ \____/  |_| \_|\____/  |_|    |______|_____/_____|  |_|
//    _______ _    _ _____  _____   ______ _____ _      ______
//   |__   __| |  | |_   _|/ ____| |  ____|_   _| |    |  ____|
//      | |  | |__| | | | | (___   | |__    | | | |    | |__
//      | |  |  __  | | |  \___ \  |  __|   | | | |    |  __|
//      | |  | |  | |_| |_ ____) | | |     _| |_| |____| |____
//      |_|  |_|  |_|_____|_____/  |_|    |_____|______|______|
//
// This file was auto-generated from a board spec definition.

use jumperless_types::{board_spec::NodePort, ChipId, Dimension, Lane, Node as NodeTrait, Port};
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Node {
    GND = 0u8,
    SUPPLY_5v = 1u8,
    _1 = 2u8,
    _2 = 3u8,
    _3 = 4u8,
    _4 = 5u8,
    _5 = 6u8,
    _6 = 7u8,
    _7 = 8u8,
    _8 = 9u8,
    _9 = 10u8,
    _10 = 11u8,
    _11 = 12u8,
    _12 = 13u8,
    _13 = 14u8,
    _14 = 15u8,
    _15 = 16u8,
    _16 = 17u8,
    _17 = 18u8,
    _18 = 19u8,
    _19 = 20u8,
    _20 = 21u8,
    _21 = 22u8,
    _22 = 23u8,
    _23 = 24u8,
    _24 = 25u8,
    _25 = 26u8,
    _26 = 27u8,
    _27 = 28u8,
    _28 = 29u8,
    _29 = 30u8,
    _30 = 31u8,
    _31 = 32u8,
    _32 = 33u8,
    _33 = 34u8,
    _34 = 35u8,
    _35 = 36u8,
    _36 = 37u8,
    _37 = 38u8,
    _38 = 39u8,
    _39 = 40u8,
    _40 = 41u8,
    _41 = 42u8,
    _42 = 43u8,
    _43 = 44u8,
    _44 = 45u8,
    _45 = 46u8,
    _46 = 47u8,
    _47 = 48u8,
    _48 = 49u8,
    _49 = 50u8,
    _50 = 51u8,
    _51 = 52u8,
    _52 = 53u8,
    _53 = 54u8,
    _54 = 55u8,
    _55 = 56u8,
    _56 = 57u8,
    _57 = 58u8,
    _58 = 59u8,
    _59 = 60u8,
    _60 = 61u8,
    NANO_D0 = 62u8,
    NANO_D1 = 63u8,
    NANO_D2 = 64u8,
    NANO_D3 = 65u8,
    NANO_D4 = 66u8,
    NANO_D5 = 67u8,
    NANO_D6 = 68u8,
    NANO_D7 = 69u8,
    NANO_D8 = 70u8,
    NANO_D9 = 71u8,
    NANO_D10 = 72u8,
    NANO_D11 = 73u8,
    NANO_D12 = 74u8,
    NANO_D13 = 75u8,
    NANO_A0 = 76u8,
    NANO_A1 = 77u8,
    NANO_A2 = 78u8,
    NANO_A3 = 79u8,
    NANO_A4 = 80u8,
    NANO_A5 = 81u8,
    NANO_A6 = 82u8,
    NANO_A7 = 83u8,
    RP_UART_RX = 84u8,
    RP_UART_TX = 85u8,
    ISENSE_PLUS = 86u8,
    ISENSE_MINUS = 87u8,
    TOP_RAIL = 88u8,
    BOTTOM_RAIL = 89u8,
    DAC1 = 90u8,
    DAC0 = 91u8,
    ADC0 = 92u8,
    ADC1 = 93u8,
    ADC2 = 94u8,
    ADC3 = 95u8,
    NANO_RESET_J0 = 96u8,
    NANO_RESET_J1 = 97u8,
    NANO_AREF = 98u8,
    GPIO_20 = 99u8,
    GPIO_21 = 100u8,
    GPIO_22 = 101u8,
    GPIO_23 = 102u8,
    GPIO_MCP_0 = 103u8,
    GPIO_MCP_1 = 104u8,
    GPIO_MCP_2 = 105u8,
    GPIO_MCP_3 = 106u8,
}
impl NodeTrait for Node {
    fn id(&self) -> u8 {
        *self as u8
    }
    fn from_id(id: u8) -> Self {
        if id >= 107u8 {
            panic!("node id out of range");
        }
        unsafe { std::mem::transmute(id) }
    }
}
impl Node {
    pub fn as_str(&self) -> &'static str {
        match self {
            Node::GND => "GND",
            Node::SUPPLY_5v => "SUPPLY_5v",
            Node::_1 => "1",
            Node::_2 => "2",
            Node::_3 => "3",
            Node::_4 => "4",
            Node::_5 => "5",
            Node::_6 => "6",
            Node::_7 => "7",
            Node::_8 => "8",
            Node::_9 => "9",
            Node::_10 => "10",
            Node::_11 => "11",
            Node::_12 => "12",
            Node::_13 => "13",
            Node::_14 => "14",
            Node::_15 => "15",
            Node::_16 => "16",
            Node::_17 => "17",
            Node::_18 => "18",
            Node::_19 => "19",
            Node::_20 => "20",
            Node::_21 => "21",
            Node::_22 => "22",
            Node::_23 => "23",
            Node::_24 => "24",
            Node::_25 => "25",
            Node::_26 => "26",
            Node::_27 => "27",
            Node::_28 => "28",
            Node::_29 => "29",
            Node::_30 => "30",
            Node::_31 => "31",
            Node::_32 => "32",
            Node::_33 => "33",
            Node::_34 => "34",
            Node::_35 => "35",
            Node::_36 => "36",
            Node::_37 => "37",
            Node::_38 => "38",
            Node::_39 => "39",
            Node::_40 => "40",
            Node::_41 => "41",
            Node::_42 => "42",
            Node::_43 => "43",
            Node::_44 => "44",
            Node::_45 => "45",
            Node::_46 => "46",
            Node::_47 => "47",
            Node::_48 => "48",
            Node::_49 => "49",
            Node::_50 => "50",
            Node::_51 => "51",
            Node::_52 => "52",
            Node::_53 => "53",
            Node::_54 => "54",
            Node::_55 => "55",
            Node::_56 => "56",
            Node::_57 => "57",
            Node::_58 => "58",
            Node::_59 => "59",
            Node::_60 => "60",
            Node::NANO_D0 => "NANO_D0",
            Node::NANO_D1 => "NANO_D1",
            Node::NANO_D2 => "NANO_D2",
            Node::NANO_D3 => "NANO_D3",
            Node::NANO_D4 => "NANO_D4",
            Node::NANO_D5 => "NANO_D5",
            Node::NANO_D6 => "NANO_D6",
            Node::NANO_D7 => "NANO_D7",
            Node::NANO_D8 => "NANO_D8",
            Node::NANO_D9 => "NANO_D9",
            Node::NANO_D10 => "NANO_D10",
            Node::NANO_D11 => "NANO_D11",
            Node::NANO_D12 => "NANO_D12",
            Node::NANO_D13 => "NANO_D13",
            Node::NANO_A0 => "NANO_A0",
            Node::NANO_A1 => "NANO_A1",
            Node::NANO_A2 => "NANO_A2",
            Node::NANO_A3 => "NANO_A3",
            Node::NANO_A4 => "NANO_A4",
            Node::NANO_A5 => "NANO_A5",
            Node::NANO_A6 => "NANO_A6",
            Node::NANO_A7 => "NANO_A7",
            Node::RP_UART_RX => "RP_UART_RX",
            Node::RP_UART_TX => "RP_UART_TX",
            Node::ISENSE_PLUS => "ISENSE_PLUS",
            Node::ISENSE_MINUS => "ISENSE_MINUS",
            Node::TOP_RAIL => "TOP_RAIL",
            Node::BOTTOM_RAIL => "BOTTOM_RAIL",
            Node::DAC1 => "DAC1",
            Node::DAC0 => "DAC0",
            Node::ADC0 => "ADC0",
            Node::ADC1 => "ADC1",
            Node::ADC2 => "ADC2",
            Node::ADC3 => "ADC3",
            Node::NANO_RESET_J0 => "NANO_RESET_J0",
            Node::NANO_RESET_J1 => "NANO_RESET_J1",
            Node::NANO_AREF => "NANO_AREF",
            Node::GPIO_20 => "GPIO_20",
            Node::GPIO_21 => "GPIO_21",
            Node::GPIO_22 => "GPIO_22",
            Node::GPIO_23 => "GPIO_23",
            Node::GPIO_MCP_0 => "GPIO_MCP_0",
            Node::GPIO_MCP_1 => "GPIO_MCP_1",
            Node::GPIO_MCP_2 => "GPIO_MCP_2",
            Node::GPIO_MCP_3 => "GPIO_MCP_3",
        }
    }
}
pub type BoardSpec = jumperless_types::board_spec::BoardSpec<Node, 108usize, 86usize, 8usize>;
pub type Board = jumperless_types::board::Board<Node, 108usize, 86usize, 8usize>;
pub fn board_spec() -> BoardSpec {
    BoardSpec {
        node_ports: [
            NodePort(
                Node::GND,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 15u8),
            ),
            NodePort(
                Node::GND,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 15u8),
            ),
            NodePort(
                Node::SUPPLY_5v,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 0u8),
            ),
            NodePort(
                Node::_1,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 0u8),
            ),
            NodePort(
                Node::_2,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_3,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_4,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_5,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_6,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_7,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_8,
                Port::new(ChipId::from_ascii(65u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_9,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_10,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_11,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_12,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_13,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_14,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_15,
                Port::new(ChipId::from_ascii(66u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_16,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_17,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_18,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_19,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_20,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_21,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_22,
                Port::new(ChipId::from_ascii(67u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_23,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_24,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_25,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_26,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_27,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_28,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_29,
                Port::new(ChipId::from_ascii(68u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_30,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 1u8),
            ),
            NodePort(
                Node::_31,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 2u8),
            ),
            NodePort(
                Node::_32,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_33,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_34,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_35,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_36,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_37,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_38,
                Port::new(ChipId::from_ascii(69u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_39,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_40,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_41,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_42,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_43,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_44,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_45,
                Port::new(ChipId::from_ascii(70u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_46,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_47,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_48,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_49,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_50,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_51,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_52,
                Port::new(ChipId::from_ascii(71u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_53,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 1u8),
            ),
            NodePort(
                Node::_54,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 2u8),
            ),
            NodePort(
                Node::_55,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 3u8),
            ),
            NodePort(
                Node::_56,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 4u8),
            ),
            NodePort(
                Node::_57,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 5u8),
            ),
            NodePort(
                Node::_58,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 6u8),
            ),
            NodePort(
                Node::_59,
                Port::new(ChipId::from_ascii(72u8), Dimension::Y, 7u8),
            ),
            NodePort(
                Node::_60,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 3u8),
            ),
            NodePort(
                Node::NANO_D0,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 0u8),
            ),
            NodePort(
                Node::NANO_D1,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 1u8),
            ),
            NodePort(
                Node::NANO_D2,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 2u8),
            ),
            NodePort(
                Node::NANO_D3,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 3u8),
            ),
            NodePort(
                Node::NANO_D4,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 4u8),
            ),
            NodePort(
                Node::NANO_D5,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 5u8),
            ),
            NodePort(
                Node::NANO_D6,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 6u8),
            ),
            NodePort(
                Node::NANO_D7,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 7u8),
            ),
            NodePort(
                Node::NANO_D8,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 8u8),
            ),
            NodePort(
                Node::NANO_D9,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 9u8),
            ),
            NodePort(
                Node::NANO_D10,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 9u8),
            ),
            NodePort(
                Node::NANO_D11,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 8u8),
            ),
            NodePort(
                Node::NANO_D12,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 10u8),
            ),
            NodePort(
                Node::NANO_D13,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 10u8),
            ),
            NodePort(
                Node::NANO_A0,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 0u8),
            ),
            NodePort(
                Node::NANO_A1,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 1u8),
            ),
            NodePort(
                Node::NANO_A2,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 2u8),
            ),
            NodePort(
                Node::NANO_A3,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 3u8),
            ),
            NodePort(
                Node::NANO_A4,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 4u8),
            ),
            NodePort(
                Node::NANO_A5,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 5u8),
            ),
            NodePort(
                Node::NANO_A6,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 6u8),
            ),
            NodePort(
                Node::NANO_A7,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 7u8),
            ),
            NodePort(
                Node::RP_UART_RX,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 11u8),
            ),
            NodePort(
                Node::RP_UART_TX,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 11u8),
            ),
            NodePort(
                Node::ISENSE_PLUS,
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 15u8),
            ),
            NodePort(
                Node::ISENSE_MINUS,
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 15u8),
            ),
            NodePort(
                Node::TOP_RAIL,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 4u8),
            ),
            NodePort(
                Node::BOTTOM_RAIL,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 5u8),
            ),
            NodePort(
                Node::DAC1,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 6u8),
            ),
            NodePort(
                Node::DAC0,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 7u8),
            ),
            NodePort(
                Node::ADC0,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 8u8),
            ),
            NodePort(
                Node::ADC1,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 9u8),
            ),
            NodePort(
                Node::ADC2,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 10u8),
            ),
            NodePort(
                Node::ADC3,
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 11u8),
            ),
            NodePort(
                Node::NANO_RESET_J0,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 1u8),
            ),
            NodePort(
                Node::NANO_RESET_J1,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 2u8),
            ),
            NodePort(
                Node::NANO_AREF,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 3u8),
            ),
            NodePort(
                Node::GPIO_20,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 4u8),
            ),
            NodePort(
                Node::GPIO_21,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 5u8),
            ),
            NodePort(
                Node::GPIO_22,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 6u8),
            ),
            NodePort(
                Node::GPIO_23,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 7u8),
            ),
            NodePort(
                Node::GPIO_MCP_0,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 8u8),
            ),
            NodePort(
                Node::GPIO_MCP_1,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 9u8),
            ),
            NodePort(
                Node::GPIO_MCP_2,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 10u8),
            ),
            NodePort(
                Node::GPIO_MCP_3,
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 11u8),
            ),
        ],
        lanes: [
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 0u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 1u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 2u8),
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 3u8),
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 4u8),
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 5u8),
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 6u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 8u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 9u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 0u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(65u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 2u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 3u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 4u8),
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 5u8),
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 6u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 8u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 9u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(66u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 1u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 4u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 5u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 6u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 8u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 9u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 2u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(67u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 6u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 8u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 9u8),
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(68u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 3u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 1u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 5u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 8u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 9u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 4u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 8u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 9u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 8u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 9u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 8u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(69u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 9u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 3u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 10u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 11u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 5u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 10u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 11u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 10u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(70u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 11u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 1u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 5u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 6u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 12u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(71u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 13u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 3u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::Y, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 7u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::Y, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(73u8), Dimension::Y, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(72u8), Dimension::X, 15u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::Y, 7u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 12u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 13u8),
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 13u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(73u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 13u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 13u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(74u8), Dimension::X, 14u8),
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 14u8),
            ),
            Lane(
                Port::new(ChipId::from_ascii(75u8), Dimension::X, 12u8),
                Port::new(ChipId::from_ascii(76u8), Dimension::X, 14u8),
            ),
        ],
        bounce_ports: [
            Port::new(ChipId::from_ascii(65u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(66u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(67u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(68u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(69u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(70u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(71u8), Dimension::Y, 0u8),
            Port::new(ChipId::from_ascii(72u8), Dimension::Y, 0u8),
        ],
    }
}
pub fn init_board() -> Board {
    Board::new(board_spec())
}
