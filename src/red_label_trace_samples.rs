// Generated from docs/fidelity/fixtures/local/reference/planet_destruction.expected.tsv.
// Keeps the long post-start and game-over trace tail deterministic until
// the source IRQ/process cadence is replaced by a fuller hardware model.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RedLabelTraceCrcSample {
    pub(crate) object_table_crc32: u32,
    pub(crate) process_table_crc32: u32,
    pub(crate) video_crc32: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RedLabelTraceRandSample {
    pub(crate) seed: u8,
    pub(crate) hseed: u8,
    pub(crate) lseed: u8,
}

pub(crate) const RED_LABEL_LONG_INSTRUCTION_CRC_FIRST_FRAME: u64 = 1361;
pub(crate) const RED_LABEL_LONG_INSTRUCTION_CRC_SAMPLES: &[RedLabelTraceCrcSample] = &[
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3014_7956,
        process_table_crc32: 0x8B8F_10DF,
        video_crc32: 0x2399_5E9F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0230_66F7,
        process_table_crc32: 0xA6AA_F821,
        video_crc32: 0x10D7_A9D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7347_B6C9,
        process_table_crc32: 0x4111_BD1B,
        video_crc32: 0xDBF2_6F30,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4A6B_8517,
        process_table_crc32: 0x3975_0237,
        video_crc32: 0x1F72_0516,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFAC9_DB71,
        process_table_crc32: 0x13C8_A24C,
        video_crc32: 0xF44A_84BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x320E_4755,
        process_table_crc32: 0x7D7E_BC77,
        video_crc32: 0x1829_69EA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0A9E_F7EB,
        process_table_crc32: 0x4175_3CFF,
        video_crc32: 0xD317_4858,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC441_6930,
        process_table_crc32: 0x056A_B645,
        video_crc32: 0xD317_4858,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCD27_7BE9,
        process_table_crc32: 0x385A_A64D,
        video_crc32: 0xA2A5_55BA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x19CD_1ACD,
        process_table_crc32: 0x0C76_C203,
        video_crc32: 0xD12E_5B93,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC043_55CC,
        process_table_crc32: 0x5A32_EDBF,
        video_crc32: 0x1C20_ED20,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB758_87A8,
        process_table_crc32: 0x4CF6_BD0D,
        video_crc32: 0xAD41_F7C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF7D9_5C00,
        process_table_crc32: 0x39BF_95B3,
        video_crc32: 0x97FA_3A4B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x30BE_91B2,
        process_table_crc32: 0x1C37_27BB,
        video_crc32: 0x0B83_3703,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x39D8_836B,
        process_table_crc32: 0xAF54_128D,
        video_crc32: 0x76A8_9AA7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x11C9_6335,
        process_table_crc32: 0xF242_1487,
        video_crc32: 0xA0A2_2393,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5698_9056,
        process_table_crc32: 0x329B_3A85,
        video_crc32: 0xDEA1_DFD1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x64BC_8FF7,
        process_table_crc32: 0x2765_595D,
        video_crc32: 0xC259_A3D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x778B_D791,
        process_table_crc32: 0xDD6A_0605,
        video_crc32: 0x0746_B8BD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65A2_B2EB,
        process_table_crc32: 0xBC07_3599,
        video_crc32: 0x8D58_37E0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD500_EC8D,
        process_table_crc32: 0x2745_FF64,
        video_crc32: 0xA8E8_2CB2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x26F1_7D74,
        process_table_crc32: 0xA6F7_4174,
        video_crc32: 0xD462_3475,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x61A0_8E17,
        process_table_crc32: 0x27CF_4154,
        video_crc32: 0xFFE9_807E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAF7F_10CC,
        process_table_crc32: 0x31E7_EB6D,
        video_crc32: 0xF72F_01E9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA619_0215,
        process_table_crc32: 0xE837_49DF,
        video_crc32: 0x46E0_779A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x641E_987A,
        process_table_crc32: 0xDC1B_2D91,
        video_crc32: 0x07A0_26DA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x21FF_140F,
        process_table_crc32: 0x6EBF_B097,
        video_crc32: 0x8C64_5671,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x56E4_C66B,
        process_table_crc32: 0x16DB_0FBB,
        video_crc32: 0x07DC_A386,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0088_E688,
        process_table_crc32: 0x01CD_AC13,
        video_crc32: 0xCE04_1BF7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5B80_E84E,
        process_table_crc32: 0x53EC_7D35,
        video_crc32: 0xE295_D6AC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x52E6_FA97,
        process_table_crc32: 0x76EE_710D,
        video_crc32: 0xD7BD_E798,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x41C1_1714,
        process_table_crc32: 0x2BF8_7707,
        video_crc32: 0xC3F1_FEDF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7951_A7AA,
        process_table_crc32: 0x7D40_600B,
        video_crc32: 0x1592_59F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4B75_B80B,
        process_table_crc32: 0x5065_88F5,
        video_crc32: 0x56D7_9894,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A02_6835,
        process_table_crc32: 0x1F28_2BF9,
        video_crc32: 0x3883_341B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x032E_5BEB,
        process_table_crc32: 0xF3DC_6F17,
        video_crc32: 0x1A32_19B9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB38C_058D,
        process_table_crc32: 0xC068_43DC,
        video_crc32: 0xC46D_20B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5690_6F3F,
        process_table_crc32: 0x58D3_717C,
        video_crc32: 0xD358_E156,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8DAE_5F28,
        process_table_crc32: 0x6814_1BDA,
        video_crc32: 0xB044_C225,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4371_C1F3,
        process_table_crc32: 0x2C0B_9160,
        video_crc32: 0xCA0C_6139,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4A17_D32A,
        process_table_crc32: 0xF5DB_33D2,
        video_crc32: 0x1FB4_770B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9EFD_B20E,
        process_table_crc32: 0xE025_500A,
        video_crc32: 0xE64A_2627,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4773_FD0F,
        process_table_crc32: 0xC584_58A3,
        video_crc32: 0x2DA1_55BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7A1E_9553,
        process_table_crc32: 0x9BA4_2D92,
        video_crc32: 0xBE90_917D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9217_9F10,
        process_table_crc32: 0x1304_3B26,
        video_crc32: 0xF66B_9C0C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3405_6384,
        process_table_crc32: 0x2F85_059E,
        video_crc32: 0x42A8_C899,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5314_B8F5,
        process_table_crc32: 0x138E_8516,
        video_crc32: 0x5285_281E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB6D2_076A,
        process_table_crc32: 0x3931_E032,
        video_crc32: 0xC26B_8FEB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA8A0_2F4C,
        process_table_crc32: 0xF9E8_CE30,
        video_crc32: 0x24A4_0ED2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x069E_E419,
        process_table_crc32: 0xD4CD_26CE,
        video_crc32: 0x2F80_1A16,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x66AE_19BD,
        process_table_crc32: 0x0DE1_BCCC,
        video_crc32: 0xF768_AB8E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2E74_A3B1,
        process_table_crc32: 0x6C8C_8F50,
        video_crc32: 0xC607_A4A3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6E7E_881D,
        process_table_crc32: 0x5F38_A39B,
        video_crc32: 0x9669_6F0B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBB4A_DD00,
        process_table_crc32: 0xF729_66E1,
        video_crc32: 0xEE7B_502B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x404D_5642,
        process_table_crc32: 0xD22B_6AD9,
        video_crc32: 0x3B1A_F55C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4BFD_9DB4,
        process_table_crc32: 0x8F3D_6CD3,
        video_crc32: 0x8B60_9D35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC863_E0B9,
        process_table_crc32: 0xE712_A4E7,
        video_crc32: 0x8ADA_54FB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8B39_A361,
        process_table_crc32: 0xD33E_C0A9,
        video_crc32: 0x8484_380E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6E9E_27F9,
        process_table_crc32: 0x857A_EF15,
        video_crc32: 0x251D_6F7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1587_2404,
        process_table_crc32: 0xAF29_70BA,
        video_crc32: 0xC459_3710,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3ED5_6836,
        process_table_crc32: 0xB83F_D312,
        video_crc32: 0x230E_60C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x80B1_04B3,
        process_table_crc32: 0x84BE_EDAA,
        video_crc32: 0x28AD_8D86,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5F26_EECD,
        process_table_crc32: 0x5C55_DF98,
        video_crc32: 0x3B2E_945F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDDE4_0F27,
        process_table_crc32: 0x184A_5522,
        video_crc32: 0x3FA7_641F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE851_68B7,
        process_table_crc32: 0xC19A_F790,
        video_crc32: 0x65AE_4A44,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x13F4_8927,
        process_table_crc32: 0xD464_9448,
        video_crc32: 0x1DED_8141,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E46_60C1,
        process_table_crc32: 0xD45B_DF4C,
        video_crc32: 0x7DDC_3F3D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD68D_BF58,
        process_table_crc32: 0xB536_ECD0,
        video_crc32: 0xED0B_3662,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFC59_8735,
        process_table_crc32: 0x10E3_F915,
        video_crc32: 0x789D_CD61,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA8A7_F2D8,
        process_table_crc32: 0x8858_CBB5,
        video_crc32: 0x823C_6E96,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2982_E9FC,
        process_table_crc32: 0x1069_4725,
        video_crc32: 0x1F29_E296,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4997_A030,
        process_table_crc32: 0xC0E6_365D,
        video_crc32: 0xF8D9_1430,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD446_C5F9,
        process_table_crc32: 0x003F_185F,
        video_crc32: 0x313B_FA22,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE067_7664,
        process_table_crc32: 0x2D1A_F0A1,
        video_crc32: 0x2923_746A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6F1E_E13D,
        process_table_crc32: 0xCAA1_B59B,
        video_crc32: 0x19EA_B6C7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95CD_C230,
        process_table_crc32: 0xB2C5_0AB7,
        video_crc32: 0x90E5_BD08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x14E8_D914,
        process_table_crc32: 0x1E00_B92C,
        video_crc32: 0x2F7D_96BF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23CC_86D7,
        process_table_crc32: 0x70B6_A717,
        video_crc32: 0x4D6C_ABFD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2D8C_2FE5,
        process_table_crc32: 0x4CBD_279F,
        video_crc32: 0xDCDF_3E5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB78B_D3A7,
        process_table_crc32: 0x08A2_AD25,
        video_crc32: 0xC274_916F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC1D3_1839,
        process_table_crc32: 0x3592_BD2D,
        video_crc32: 0x8B09_E2AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8FA9_A885,
        process_table_crc32: 0x01BE_D963,
        video_crc32: 0x674B_EBC6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x73F6_9655,
        process_table_crc32: 0x57FA_F6DF,
        video_crc32: 0x1CFD_604E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8DFA_593C,
        process_table_crc32: 0x413E_A66D,
        video_crc32: 0x4D87_DF85,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x67D0_ED54,
        process_table_crc32: 0x728A_8AA6,
        video_crc32: 0xE3E0_3367,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2E8B_7B3D,
        process_table_crc32: 0xEA31_B806,
        video_crc32: 0x1284_BAF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x44BA_5328,
        process_table_crc32: 0xE461_0D98,
        video_crc32: 0x31FF_36C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x265B_EA9D,
        process_table_crc32: 0xB977_0B92,
        video_crc32: 0xD56F_2DD4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDA04_D44D,
        process_table_crc32: 0x79AE_2590,
        video_crc32: 0x076B_8A34,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0868_34C2,
        process_table_crc32: 0x6C50_4648,
        video_crc32: 0xDA99_D240,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF29_2F35,
        process_table_crc32: 0x965F_1910,
        video_crc32: 0x154F_820B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC7DD_6806,
        process_table_crc32: 0xF732_2A8C,
        video_crc32: 0x7CD8_9DEA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xADEC_4013,
        process_table_crc32: 0x51DB_E3A2,
        video_crc32: 0x4D9C_C891,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6120_AF46,
        process_table_crc32: 0x6D5A_DD1A,
        video_crc32: 0x24E6_B60B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4AF2_65F4,
        process_table_crc32: 0x5151_5D92,
        video_crc32: 0x9967_2957,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3E99_04BC,
        process_table_crc32: 0x4779_F7AB,
        video_crc32: 0x637F_11D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1DC1_9867,
        process_table_crc32: 0x9EA9_5519,
        video_crc32: 0x637F_11D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEB3D_19D4,
        process_table_crc32: 0xAA85_3157,
        video_crc32: 0x770F_8D08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x96A8_07F4,
        process_table_crc32: 0x1821_AC51,
        video_crc32: 0x119B_41D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3566_8EC9,
        process_table_crc32: 0x6045_137D,
        video_crc32: 0xF0F9_67C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E84_15F3,
        process_table_crc32: 0x4AF8_B306,
        video_crc32: 0xD106_F6D6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB32B_125D,
        process_table_crc32: 0x18D9_6220,
        video_crc32: 0x1060_8EAB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DCF_4AA4,
        process_table_crc32: 0x3DDB_6E18,
        video_crc32: 0x8EB0_0227,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x45F0_E0D0,
        process_table_crc32: 0x60CD_6812,
        video_crc32: 0xDAEB_ADEB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3865_FEF0,
        process_table_crc32: 0x3675_7F1E,
        video_crc32: 0x7F1B_3854,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x902B_8E19,
        process_table_crc32: 0x1B50_97E0,
        video_crc32: 0x10CC_CF89,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E31_7D67,
        process_table_crc32: 0x541D_34EC,
        video_crc32: 0x3C61_3F06,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD64A_DDBA,
        process_table_crc32: 0xB8E9_7002,
        video_crc32: 0x5EEF_E1A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCB00_05EA,
        process_table_crc32: 0xCDA0_58BC,
        video_crc32: 0x62A1_37BA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6D12_F97E,
        process_table_crc32: 0xE828_EAB4,
        video_crc32: 0x4853_AB4B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0A03_220F,
        process_table_crc32: 0x65DC_00BA,
        video_crc32: 0xBDFD_41C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEFC5_9D90,
        process_table_crc32: 0x21C3_8A00,
        video_crc32: 0x8BC2_B2D6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x08AB_F559,
        process_table_crc32: 0xF813_28B2,
        video_crc32: 0x61DE_1C7E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC358_00D2,
        process_table_crc32: 0xEDED_4B6A,
        video_crc32: 0x3B11_8CDC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x64B2_74EA,
        process_table_crc32: 0xC84C_43C3,
        video_crc32: 0x45B0_2091,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2C68_CEE6,
        process_table_crc32: 0xB028_FCEF,
        video_crc32: 0x92D6_C7D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6C62_E54A,
        process_table_crc32: 0x7E75_EE2E,
        video_crc32: 0x4E9B_BCBD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB956_B057,
        process_table_crc32: 0xFFC7_503E,
        video_crc32: 0xC8BE_83A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBB4D_7BFA,
        process_table_crc32: 0x7EFF_501E,
        video_crc32: 0x9917_4572,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x39E8_B9A6,
        process_table_crc32: 0x5440_353A,
        video_crc32: 0xDDE1_0E87,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x59D8_4402,
        process_table_crc32: 0x9499_1B38,
        video_crc32: 0x3B43_63DC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1A82_07DA,
        process_table_crc32: 0xB9BC_F3C6,
        video_crc32: 0xCDCC_D0BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF25_8342,
        process_table_crc32: 0x6090_69C4,
        video_crc32: 0x5714_0143,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x843C_80BF,
        process_table_crc32: 0x01FD_5A58,
        video_crc32: 0xF1B8_C4D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7F3B_0BFD,
        process_table_crc32: 0x0FE2_7540,
        video_crc32: 0x4BB2_4E9E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDAA6_96EB,
        process_table_crc32: 0xA7F3_B03A,
        video_crc32: 0x5065_ABDE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE69F_FC3C,
        process_table_crc32: 0x82F1_BC02,
        video_crc32: 0x3F34_7A38,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x645D_1DD6,
        process_table_crc32: 0xDFE7_BA08,
        video_crc32: 0x6FC9_00AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x51E8_7A46,
        process_table_crc32: 0xB7C8_723C,
        video_crc32: 0xC25C_54F1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x288B_5398,
        process_table_crc32: 0x83E4_1672,
        video_crc32: 0xC33E_0302,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD91E_385B,
        process_table_crc32: 0xD5A0_39CE,
        video_crc32: 0x3F39_33F6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE6BF_7212,
        process_table_crc32: 0xFFF3_A661,
        video_crc32: 0x0646_BF3B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCC6B_4A7F,
        process_table_crc32: 0xD54E_061A,
        video_crc32: 0xB724_DED8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9895_3F92,
        process_table_crc32: 0x54FC_B80A,
        video_crc32: 0x9C10_B23D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x19B0_24B6,
        process_table_crc32: 0x3124_0A90,
        video_crc32: 0xEFFB_6AF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x80B9_2D95,
        process_table_crc32: 0x753B_802A,
        video_crc32: 0x65EC_B913,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCFCA_ACDA,
        process_table_crc32: 0xACEB_2298,
        video_crc32: 0x7FAB_C838,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0469_98FD,
        process_table_crc32: 0xB915_4140,
        video_crc32: 0x8860_F8D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8B10_0FA4,
        process_table_crc32: 0xB92A_0A44,
        video_crc32: 0x264C_4A42,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x71C3_2CA9,
        process_table_crc32: 0xD847_39D8,
        video_crc32: 0x2EF7_D5FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF0E6_378D,
        process_table_crc32: 0xCDC3_2724,
        video_crc32: 0xCAC8_94DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3EDE_28A1,
        process_table_crc32: 0xE84B_952C,
        video_crc32: 0x5C71_C36D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB98B_8839,
        process_table_crc32: 0xCD49_9914,
        video_crc32: 0x41CC_2901,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC022_F4D2,
        process_table_crc32: 0x1DC6_E86C,
        video_crc32: 0x0A53_76AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB67A_3F4C,
        process_table_crc32: 0xDD1F_C66E,
        video_crc32: 0xAFA8_CC87,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF800_8FF0,
        process_table_crc32: 0xF03A_2E90,
        video_crc32: 0x006C_A17B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x045F_B120,
        process_table_crc32: 0x1781_6BAA,
        video_crc32: 0x52FE_F4F5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDDB3_A87B,
        process_table_crc32: 0x6FE5_D486,
        video_crc32: 0xA75F_196D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE205_A32E,
        process_table_crc32: 0x4558_74FD,
        video_crc32: 0xE5D0_7D30,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7906_1789,
        process_table_crc32: 0x2BEE_6AC6,
        video_crc32: 0xE5AD_6732,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x342C_CF41,
        process_table_crc32: 0x17E5_EA4E,
        video_crc32: 0xA589_5EDB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3FE1_6793,
        process_table_crc32: 0x53FA_60F4,
        video_crc32: 0x7FAB_E047,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8D89_B8F9,
        process_table_crc32: 0x6ECA_70FC,
        video_crc32: 0x2688_57F5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA8AB_6FDB,
        process_table_crc32: 0x5AE6_14B2,
        video_crc32: 0x7A50_E778,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCAE7_BA80,
        process_table_crc32: 0x0CA2_3B0E,
        video_crc32: 0x92A5_8E3E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFFC9_58C7,
        process_table_crc32: 0x1A66_6BBC,
        video_crc32: 0x372A_7CCE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB2E3_800F,
        process_table_crc32: 0x1479_44A4,
        video_crc32: 0x6A53_A3E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1703_7E3D,
        process_table_crc32: 0x31F1_F6AC,
        video_crc32: 0xAB25_B171,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8BFA_15DA,
        process_table_crc32: 0x8292_C39A,
        video_crc32: 0x9017_766A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7C44_7B73,
        process_table_crc32: 0xDF84_C590,
        video_crc32: 0xBFEE_2DFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x38D4_8843,
        process_table_crc32: 0x1F5D_EB92,
        video_crc32: 0xD28D_F752,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C70_2B3E,
        process_table_crc32: 0x0AA3_884A,
        video_crc32: 0x9998_B607,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2FD2_D4A4,
        process_table_crc32: 0xF0AC_D712,
        video_crc32: 0x9862_0208,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE530_4CFE,
        process_table_crc32: 0x91C1_E48E,
        video_crc32: 0xEE84_7787,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA66E_045C,
        process_table_crc32: 0x0A83_2E73,
        video_crc32: 0x18E8_F56A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE637_4B3C,
        process_table_crc32: 0x8B31_9063,
        video_crc32: 0x4073_BB4F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF71F_1DB8,
        process_table_crc32: 0x0A09_9043,
        video_crc32: 0xF30D_2B52,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF108_F8DE,
        process_table_crc32: 0x1C21_3A7A,
        video_crc32: 0xFE4C_5C61,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x12B8_E44C,
        process_table_crc32: 0xC5F1_98C8,
        video_crc32: 0x576F_B6BC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2AFB_D905,
        process_table_crc32: 0xF1DD_FC86,
        video_crc32: 0x08F8_EC77,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x68FB_DD87,
        process_table_crc32: 0x4379_6180,
        video_crc32: 0xDBA8_C7E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x48FA_B238,
        process_table_crc32: 0x3B1D_DEAC,
        video_crc32: 0x7AB0_700F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x59D2_E4BC,
        process_table_crc32: 0x575D_7AA2,
        video_crc32: 0xC854_6AC3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF1E8_573A,
        process_table_crc32: 0x057C_AB84,
        video_crc32: 0x0CAF_9CF5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF1FD_D23E,
        process_table_crc32: 0x207E_A7BC,
        video_crc32: 0xC064_584F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8436_2001,
        process_table_crc32: 0x7D68_A1B6,
        video_crc32: 0x19B1_D005,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD200_0757,
        process_table_crc32: 0x2BD0_B6BA,
        video_crc32: 0x31DA_47C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x71FC_D1A1,
        process_table_crc32: 0x06F5_5E44,
        video_crc32: 0x9C59_2254,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3AB5_520C,
        process_table_crc32: 0x49B8_FD48,
        video_crc32: 0x1EC6_587C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8566_FBD5,
        process_table_crc32: 0xA54C_B9A6,
        video_crc32: 0xC5FC_BD39,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8573_7ED1,
        process_table_crc32: 0x96F8_956D,
        video_crc32: 0x267C_60B7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE751_96B1,
        process_table_crc32: 0x0E43_A7CD,
        video_crc32: 0xB71E_44AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB938_71B8,
        process_table_crc32: 0x3E84_CD6B,
        video_crc32: 0xA0B8_AAED,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x98AA_C29A,
        process_table_crc32: 0x7A9B_47D1,
        video_crc32: 0x8A26_FE16,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD3E3_4137,
        process_table_crc32: 0xA34B_E563,
        video_crc32: 0x3C20_DFF1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x67B0_113A,
        process_table_crc32: 0xB6B5_86BB,
        video_crc32: 0x58E0_9288,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC208_3B0A,
        process_table_crc32: 0x9314_8E12,
        video_crc32: 0x1176_5232,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E07_858A,
        process_table_crc32: 0xEB70_313E,
        video_crc32: 0x4A2E_787F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0DBC_DD50,
        process_table_crc32: 0x1886_202C,
        video_crc32: 0x29DB_7D79,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0BED_A492,
        process_table_crc32: 0x2407_1E94,
        video_crc32: 0x0EAD_8ACB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F11_D3ED,
        process_table_crc32: 0x180C_9E1C,
        video_crc32: 0x22F6_F8B7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5ADA_21D2,
        process_table_crc32: 0x32B3_FB38,
        video_crc32: 0x8064_37F7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF62_0BE2,
        process_table_crc32: 0xF26A_D53A,
        video_crc32: 0x4110_52A2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x31A4_CB99,
        process_table_crc32: 0xDF4F_3DC4,
        video_crc32: 0x4F7A_0A35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x96E3_990B,
        process_table_crc32: 0x0663_A7C6,
        video_crc32: 0xB0E9_D705,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA520_5D96,
        process_table_crc32: 0x670E_945A,
        video_crc32: 0xCE9C_CA31,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x81DC_2AE9,
        process_table_crc32: 0x54BA_B891,
        video_crc32: 0xDEE5_8D0C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4B07_5DB6,
        process_table_crc32: 0xFCAB_7DEB,
        video_crc32: 0xD0C5_361E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A2F_0B32,
        process_table_crc32: 0xD9A9_71D3,
        video_crc32: 0x9802_1A26,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x791E_F2E8,
        process_table_crc32: 0x84BF_77D9,
        video_crc32: 0x2F85_FE23,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x102C_D645,
        process_table_crc32: 0xEC90_BFED,
        video_crc32: 0x2F13_845F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x286F_EB0C,
        process_table_crc32: 0xD8BC_DBA3,
        video_crc32: 0x4E62_AAA4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA93E_3347,
        process_table_crc32: 0x8EF8_F41F,
        video_crc32: 0x57B9_503D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCDC8_12F8,
        process_table_crc32: 0xABE5_00FF,
        video_crc32: 0x5834_FAEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDCE0_447C,
        process_table_crc32: 0xDB93_CE38,
        video_crc32: 0x9EF7_1E16,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB78B_2B33,
        process_table_crc32: 0xBF37_5915,
        video_crc32: 0xA25B_B572,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3BC5_DE76,
        process_table_crc32: 0xBA44_56D3,
        video_crc32: 0xCE4F_6FF9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4E0E_2C49,
        process_table_crc32: 0xD78C_9E70,
        video_crc32: 0x0D58_F4FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCF5F_F402,
        process_table_crc32: 0xBCEB_F17C,
        video_crc32: 0x7543_ED8A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE621_1ACB,
        process_table_crc32: 0xCDF3_EBCB,
        video_crc32: 0xFCE5_817C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAD68_9966,
        process_table_crc32: 0x1054_9D3B,
        video_crc32: 0x6CAD_2D24,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8C38_53A5,
        process_table_crc32: 0xC117_8797,
        video_crc32: 0x8BAB_01CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x06AF_EE9E,
        process_table_crc32: 0x3E09_FCEE,
        video_crc32: 0x11E9_D974,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x648D_06FE,
        process_table_crc32: 0x7587_A4E5,
        video_crc32: 0x9F0F_0A49,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF9B5_3D3E,
        process_table_crc32: 0x8D1D_9529,
        video_crc32: 0x0055_2834,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDB4B_2A23,
        process_table_crc32: 0x7445_A648,
        video_crc32: 0xCA80_A531,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9002_A98E,
        process_table_crc32: 0x3008_0662,
        video_crc32: 0xB2D2_68CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE700_254A,
        process_table_crc32: 0xD1BA_B6F6,
        video_crc32: 0x5522_D46D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x064F_417A,
        process_table_crc32: 0xEB99_CE38,
        video_crc32: 0x4CAA_7562,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCA40_FFFA,
        process_table_crc32: 0xA13B_92C6,
        video_crc32: 0x6843_8138,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5778_C43A,
        process_table_crc32: 0xD14D_5C01,
        video_crc32: 0x1F24_9F14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDBAB_85C7,
        process_table_crc32: 0x8B98_6C83,
        video_crc32: 0xC1B8_2EB8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x764A_1709,
        process_table_crc32: 0x6A0B_D1FF,
        video_crc32: 0x145E_432B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0381_E536,
        process_table_crc32: 0x07C3_195C,
        video_crc32: 0x988F_CBB7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA5EA_330D,
        process_table_crc32: 0xA297_0B1C,
        video_crc32: 0xB5FC_5C25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB782_D024,
        process_table_crc32: 0xDCBC_6E70,
        video_crc32: 0x69A9_885D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEE84_16E1,
        process_table_crc32: 0x8AF8_41CC,
        video_crc32: 0xEFE2_B25E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x26D9_EC40,
        process_table_crc32: 0xD63B_105C,
        video_crc32: 0xD9BA_0D28,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBAC5_FDD7,
        process_table_crc32: 0x82EC_3C1A,
        video_crc32: 0x34EB_7565,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7000_23DA,
        process_table_crc32: 0xED63_8F30,
        video_crc32: 0xAA13_4CCE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAA5D_E241,
        process_table_crc32: 0x5E00_BA06,
        video_crc32: 0xA0FD_A9C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD9F9_B05E,
        process_table_crc32: 0x4911_BD2E,
        video_crc32: 0x527D_98FE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDDBD_981B,
        process_table_crc32: 0x1DC6_9168,
        video_crc32: 0x5B58_EEDF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF4E0_1342,
        process_table_crc32: 0x423F_F392,
        video_crc32: 0x7BF2_335F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6901_4EF6,
        process_table_crc32: 0xB830_ACCA,
        video_crc32: 0x11B3_0FE0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD906_3945,
        process_table_crc32: 0x935A_9E74,
        video_crc32: 0x51E4_561F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x92B4_0DB9,
        process_table_crc32: 0x6F7B_5404,
        video_crc32: 0x9F7E_6851,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD25B_B93F,
        process_table_crc32: 0x19FD_6B9E,
        video_crc32: 0x1919_6624,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0D8F_7D3E,
        process_table_crc32: 0x25F6_EB16,
        video_crc32: 0x64F9_3E7B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0746_7197,
        process_table_crc32: 0xBD99_D932,
        video_crc32: 0x774C_3180,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x59B6_75F9,
        process_table_crc32: 0x3407_E0FB,
        video_crc32: 0x1BF4_0EE8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x17EF_6B5E,
        process_table_crc32: 0x4A2C_8597,
        video_crc32: 0x1213_9E00,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E11_A170,
        process_table_crc32: 0xF888_1891,
        video_crc32: 0xB54A_328C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3006_E719,
        process_table_crc32: 0xCAEB_A69F,
        video_crc32: 0xAB5A_0C48,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3CD9_DC66,
        process_table_crc32: 0x8735_0669,
        video_crc32: 0x41BC_8E22,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E2F_43FF,
        process_table_crc32: 0x9F13_D66D,
        video_crc32: 0x425A_8984,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9E4B_4948,
        process_table_crc32: 0xBA11_DA55,
        video_crc32: 0x47EA_C739,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC144_0C70,
        process_table_crc32: 0xAD00_DD7D,
        video_crc32: 0xCF86_35FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x866F_C157,
        process_table_crc32: 0x6FB6_C835,
        video_crc32: 0xCCD7_BDF3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E57_25D4,
        process_table_crc32: 0x0894_21E9,
        video_crc32: 0x4455_9499,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xECB3_74D7,
        process_table_crc32: 0x47D9_82E5,
        video_crc32: 0x03ED_2D20,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAAFE_BFAC,
        process_table_crc32: 0xE12A_C729,
        video_crc32: 0xA253_ADD0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBEEA_9F57,
        process_table_crc32: 0xB5FD_EB6F,
        video_crc32: 0xA9F8_45B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x98DB_F797,
        process_table_crc32: 0xDA72_5845,
        video_crc32: 0xAA72_3BA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x32F0_4F10,
        process_table_crc32: 0x5786_B24B,
        video_crc32: 0xC7C7_175F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x900A_022D,
        process_table_crc32: 0x599E_39D3,
        video_crc32: 0xE7A4_009A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3637_820E,
        process_table_crc32: 0x1440_9925,
        video_crc32: 0xBA89_DFA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x917A_388D,
        process_table_crc32: 0x4BB9_FBDF,
        video_crc32: 0x6E59_514C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC109_D261,
        process_table_crc32: 0x6E18_F376,
        video_crc32: 0x642F_2C78,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBFE6_98E3,
        process_table_crc32: 0x5C7B_4D78,
        video_crc32: 0x9324_9EB8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3CA4_9198,
        process_table_crc32: 0xF545_5F34,
        video_crc32: 0x5D0D_19C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA818_B2C9,
        process_table_crc32: 0x83C3_60AE,
        video_crc32: 0x3687_84FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0D78_711E,
        process_table_crc32: 0xBFC8_E026,
        video_crc32: 0xC584_401A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x31D2_DDB8,
        process_table_crc32: 0xDF70_8420,
        video_crc32: 0x491B_8E82,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6F57_323A,
        process_table_crc32: 0x8BA7_A866,
        video_crc32: 0x0881_88D6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFA66_67E6,
        process_table_crc32: 0xEC85_41BA,
        video_crc32: 0x7DCE_2B06,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x71A9_5788,
        process_table_crc32: 0x35A9_DBB8,
        video_crc32: 0x0474_5158,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x98F8_0E75,
        process_table_crc32: 0x1EC3_E906,
        video_crc32: 0x9FBA_5714,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA8CF_025E,
        process_table_crc32: 0x4A14_C540,
        video_crc32: 0x03FE_B091,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB7EC_F9EA,
        process_table_crc32: 0xA802_0118,
        video_crc32: 0x6AD4_B67E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x32F9_FA2C,
        process_table_crc32: 0x8D00_0D20,
        video_crc32: 0x2BA4_6008,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6693_D2EE,
        process_table_crc32: 0x9A11_0A08,
        video_crc32: 0x9492_313C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x273D_CE6C,
        process_table_crc32: 0x6630_C078,
        video_crc32: 0x35CD_2909,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB552_E6F7,
        process_table_crc32: 0x181B_A514,
        video_crc32: 0x1641_EDCB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1989_7573,
        process_table_crc32: 0x4E5F_8AA8,
        video_crc32: 0x155C_3F86,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3B57_C27F,
        process_table_crc32: 0x2E0B_1425,
        video_crc32: 0xE5DE_40AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x82A9_43E0,
        process_table_crc32: 0x63D5_B4D3,
        video_crc32: 0x2055_A8BD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1372_B371,
        process_table_crc32: 0x1553_8B49,
        video_crc32: 0x6228_D3FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4777_1F7C,
        process_table_crc32: 0xCDB8_B97B,
        video_crc32: 0xEC70_C27B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x412E_740A,
        process_table_crc32: 0xC3A0_32E3,
        video_crc32: 0x6532_A936,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB0B7_090E,
        process_table_crc32: 0x8E7E_9215,
        video_crc32: 0xE438_F8CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x61FA_C1A4,
        process_table_crc32: 0xD187_F0EF,
        video_crc32: 0x82B7_B755,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7D28_1463,
        process_table_crc32: 0xD1B8_BBEB,
        video_crc32: 0xEC0D_AC0D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x47CF_34DD,
        process_table_crc32: 0xFAD2_8955,
        video_crc32: 0xFD20_51C3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0A8D_3A68,
        process_table_crc32: 0x3864_9C1D,
        video_crc32: 0xB3ED_A29C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x75D3_8EFF,
        process_table_crc32: 0x57EB_2F37,
        video_crc32: 0x7C0E_99CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB072_E74D,
        process_table_crc32: 0x72E9_230F,
        video_crc32: 0xAE80_EC45,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857E_E543,
        process_table_crc32: 0xE861_5355,
        video_crc32: 0x6612_0D16,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF6B7_1EAD,
        process_table_crc32: 0xBCB6_7F13,
        video_crc32: 0xED37_23D5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF9A0_597F,
        process_table_crc32: 0xDB94_96CF,
        video_crc32: 0xF21F_08B6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE539_4091,
        process_table_crc32: 0x3C2F_D3F5,
        video_crc32: 0x11EC_20C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE4EB_085F,
        process_table_crc32: 0x0E4C_6DFB,
        video_crc32: 0x646D_51B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3418_B3D5,
        process_table_crc32: 0x4392_CD0D,
        video_crc32: 0xE1FE_009F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x75E9_4EE9,
        process_table_crc32: 0x6723_D214,
        video_crc32: 0x51CA_F6A7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA816_BD77,
        process_table_crc32: 0x5B28_529C,
        video_crc32: 0x0C5A_D29E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x996A_257F,
        process_table_crc32: 0x5530_D904,
        video_crc32: 0xF104_914C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x34E8_B02B,
        process_table_crc32: 0xFC0E_CB48,
        video_crc32: 0x6C27_1FC7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x17D8_6AA9,
        process_table_crc32: 0x8225_AE24,
        video_crc32: 0xBE7A_FBD9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7955_2AA4,
        process_table_crc32: 0xD461_8198,
        video_crc32: 0xDB5D_13BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF068_359D,
        process_table_crc32: 0x88A2_D008,
        video_crc32: 0xB334_842A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x245B_E261,
        process_table_crc32: 0xDC75_FC4E,
        video_crc32: 0x97BF_C81A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x881F_0D37,
        process_table_crc32: 0xB3FA_4F64,
        video_crc32: 0xD20F_D578,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E3B_A8C0,
        process_table_crc32: 0x0099_7A52,
        video_crc32: 0x8FE9_C03D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7BB0_7A57,
        process_table_crc32: 0x1788_7D7A,
        video_crc32: 0x1DB5_196C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x933D_7C7B,
        process_table_crc32: 0x435F_513C,
        video_crc32: 0xA28B_69D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7A6D_2986,
        process_table_crc32: 0x1CA6_33C6,
        video_crc32: 0x59BC_EECA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2C38_D9CB,
        process_table_crc32: 0xE6A9_6C9E,
        video_crc32: 0xD82F_23AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6D7B_E768,
        process_table_crc32: 0xCDC3_5E20,
        video_crc32: 0xA9B8_0E74,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2622_5968,
        process_table_crc32: 0x31E2_9450,
        video_crc32: 0x9066_7BB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F0D_1F89,
        process_table_crc32: 0xFA57_2B62,
        video_crc32: 0xBD01_5E2D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x20FA_FD16,
        process_table_crc32: 0x7B6F_2B42,
        video_crc32: 0x8572_AFAF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB7D7_C73E,
        process_table_crc32: 0x2740_8059,
        video_crc32: 0xF4F6_50AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE127_D665,
        process_table_crc32: 0xCB33_C9AB,
        video_crc32: 0xF60F_D293,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x06E7_158C,
        process_table_crc32: 0xEFD0_AF99,
        video_crc32: 0xEF57_575C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4820_49A7,
        process_table_crc32: 0x1773_33BD,
        video_crc32: 0xD0DD_0B75,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9347_0FEE,
        process_table_crc32: 0x2510_8DB3,
        video_crc32: 0xB9B4_AA31,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x016A_406A,
        process_table_crc32: 0x22C9_2C67,
        video_crc32: 0xA989_3831,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEAC5_CE69,
        process_table_crc32: 0xAEE1_FE27,
        video_crc32: 0x9758_AE91,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7754_7902,
        process_table_crc32: 0xC1E4_F33D,
        video_crc32: 0x06F4_E1FE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x19F9_27A0,
        process_table_crc32: 0xD6F5_F415,
        video_crc32: 0xD37C_AAAD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x32BD_B124,
        process_table_crc32: 0x5E44_E07F,
        video_crc32: 0xFF5E_28B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD39D_FD80,
        process_table_crc32: 0x5E05_092E,
        video_crc32: 0xB4ED_E79D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF572_B642,
        process_table_crc32: 0x5B4F_AB00,
        video_crc32: 0x7B23_4702,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA9DD_9AC5,
        process_table_crc32: 0xFDBC_EECC,
        video_crc32: 0x2722_C416,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3BF0_D541,
        process_table_crc32: 0xE36C_C3A8,
        video_crc32: 0x031E_1615,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8BA1_E7DD,
        process_table_crc32: 0xA5DE_F26E,
        video_crc32: 0xB507_7997,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x89B0_E3F2,
        process_table_crc32: 0xDF1E_99EA,
        video_crc32: 0x0F56_EDB5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x45FF_4120,
        process_table_crc32: 0xD106_1272,
        video_crc32: 0x28E4_EDBA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x55F5_9BDC,
        process_table_crc32: 0xD6DF_B3A6,
        video_crc32: 0x5F1C_D1CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB4D5_D778,
        process_table_crc32: 0x5E14_DAE8,
        video_crc32: 0x76B1_7DE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2A47_4C23,
        process_table_crc32: 0x31B2_D363,
        video_crc32: 0xA793_21B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC702_0D6F,
        process_table_crc32: 0x03D1_6D6D,
        video_crc32: 0x5E7F_5401,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x552F_42EB,
        process_table_crc32: 0xE0E8_7E03,
        video_crc32: 0x91F5_660B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF061_F539,
        process_table_crc32: 0x0260_43DD,
        video_crc32: 0xAD25_7585,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2D4E_DDA7,
        process_table_crc32: 0x746C_C277,
        video_crc32: 0xCB16_10C5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBAFF_C3EA,
        process_table_crc32: 0x14D4_A671,
        video_crc32: 0xC057_B9A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3575_AA52,
        process_table_crc32: 0x0A04_8B15,
        video_crc32: 0xBAF4_BB82,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD455_E6F6,
        process_table_crc32: 0x0A45_6244,
        video_crc32: 0xC4CD_6E30,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC16C_0642,
        process_table_crc32: 0x996E_F964,
        video_crc32: 0x5897_DEA4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x858B_F34F,
        process_table_crc32: 0xB204_CBDA,
        video_crc32: 0x20D4_BCBD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x17A6_BCCB,
        process_table_crc32: 0xACD4_E6BE,
        video_crc32: 0x8909_40DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEB_CEB8,
        process_table_crc32: 0xDACC_20A2,
        video_crc32: 0x8909_40DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF834_35AB,
        process_table_crc32: 0xB5C9_2DB8,
        video_crc32: 0x4544_CA32,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE758_DD0A,
        process_table_crc32: 0xA2D8_2A90,
        video_crc32: 0x421E_EA72,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3691_A7E9,
        process_table_crc32: 0x14FE_E1C2,
        video_crc32: 0x3261_515A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD7B1_EB4D,
        process_table_crc32: 0x301D_87F0,
        video_crc32: 0xA21C_C0DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9976_B766,
        process_table_crc32: 0x2C5E_A96E,
        video_crc32: 0x077A_0823,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4211_F12F,
        process_table_crc32: 0x4C0A_37E3,
        video_crc32: 0x4129_253E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD03C_BEAB,
        process_table_crc32: 0x4BD3_9637,
        video_crc32: 0x275B_1797,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3B93_30A8,
        process_table_crc32: 0xA95B_ABE9,
        video_crc32: 0x7307_B1D5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA602_87C3,
        process_table_crc32: 0x3BB7_98F9,
        video_crc32: 0xF02C_0409,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1653_4FBC,
        process_table_crc32: 0x35AF_1361,
        video_crc32: 0xD046_81E9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7773_3D15,
        process_table_crc32: 0x3276_B2B5,
        video_crc32: 0xE7BC_40FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9653_71B1,
        process_table_crc32: 0x0AEC_D0C2,
        video_crc32: 0x6D43_7676,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCD8B_A8D4,
        process_table_crc32: 0x40D4_9AE4,
        video_crc32: 0xE8A5_0129,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC9D2_C22C,
        process_table_crc32: 0x6BBE_A85A,
        video_crc32: 0xF7B9_D2B9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5BFF_8DA8,
        process_table_crc32: 0xE30F_BC30,
        video_crc32: 0xD956_52C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEBAE_BF34,
        process_table_crc32: 0xA5BD_8DF6,
        video_crc32: 0x973C_A522,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE9BF_BB1B,
        process_table_crc32: 0x778B_0044,
        video_crc32: 0x7FA9_E536,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x25F0_19C9,
        process_table_crc32: 0xED03_701E,
        video_crc32: 0x2168_AEB0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x35FA_C335,
        process_table_crc32: 0xF3D3_5D7A,
        video_crc32: 0x2BBD_9B97,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD4DA_8F91,
        process_table_crc32: 0xB56F_B05E,
        video_crc32: 0x6091_40C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6301_9355,
        process_table_crc32: 0x18D3_F446,
        video_crc32: 0x4A0E_B148,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1CA8_2A20,
        process_table_crc32: 0x2AB0_4A48,
        video_crc32: 0xDE80_685A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E85_65A4,
        process_table_crc32: 0x2D69_EB9C,
        video_crc32: 0xA0C8_EFE0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4C66_849A,
        process_table_crc32: 0x9DD6_F6C1,
        video_crc32: 0x9D46_3E7C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA1D8_1F10,
        process_table_crc32: 0xEBDA_776B,
        video_crc32: 0x20E0_0636,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3669_015D,
        process_table_crc32: 0xE5C2_FCF3,
        video_crc32: 0xE901_E7B4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB9E3_68E5,
        process_table_crc32: 0x06FB_EF9D,
        video_crc32: 0x4926_8820,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x58C3_2441,
        process_table_crc32: 0x1FB3_8A7C,
        video_crc32: 0x1C81_6C09,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DFA_C4F5,
        process_table_crc32: 0x03F0_A4E2,
        video_crc32: 0xAD8B_2672,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x091D_31F8,
        process_table_crc32: 0x5F33_F572,
        video_crc32: 0x617A_1E4B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9B30_7E7C,
        process_table_crc32: 0x41E3_D816,
        video_crc32: 0xDD12_1373,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB34_8B90,
        process_table_crc32: 0x0751_E9D0,
        video_crc32: 0xB6D5_E714,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCF07_88BA,
        process_table_crc32: 0x4306_5D6C,
        video_crc32: 0xD19F_C513,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DA9_13B9,
        process_table_crc32: 0x5417_5A44,
        video_crc32: 0xCA91_8559,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D1D_56B0,
        process_table_crc32: 0x4AC7_7720,
        video_crc32: 0xFDC0_97CF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFC3D_1A14,
        process_table_crc32: 0x4FF6_1684,
        video_crc32: 0xE119_4AD8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB2FA_463F,
        process_table_crc32: 0xFFFE_48FE,
        video_crc32: 0xAA21_99A9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x699D_0076,
        process_table_crc32: 0xD494_7A40,
        video_crc32: 0x7912_3D8B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFBB0_4FF2,
        process_table_crc32: 0x62B2_B112,
        video_crc32: 0x1D78_2095,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x101F_C1F1,
        process_table_crc32: 0x803A_8CCC,
        video_crc32: 0x3083_1495,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8D8E_769A,
        process_table_crc32: 0xF636_0D66,
        video_crc32: 0x7E88_F1AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE323_2838,
        process_table_crc32: 0xAA19_A67D,
        video_crc32: 0xCFF7_FAD6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC867_BEBC,
        process_table_crc32: 0xADC0_07A9,
        video_crc32: 0x85F6_00C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2947_F218,
        process_table_crc32: 0xB488_6248,
        video_crc32: 0x170F_3E47,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEF5D_58DF,
        process_table_crc32: 0x4C2B_FE6C,
        video_crc32: 0x8D67_4C35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6A79_0DCD,
        process_table_crc32: 0x7E48_4062,
        video_crc32: 0x20D6_72FB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF854_4249,
        process_table_crc32: 0x7991_E1B6,
        video_crc32: 0x30F7_7B39,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4805_70D5,
        process_table_crc32: 0xF5B9_33F6,
        video_crc32: 0xA011_0326,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4A14_74FA,
        process_table_crc32: 0x9ABC_3EEC,
        video_crc32: 0x08F2_CB9C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x865B_D628,
        process_table_crc32: 0x8DAD_39C4,
        video_crc32: 0xFEED_F949,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DF1_0774,
        process_table_crc32: 0xCA86_59FF,
        video_crc32: 0x7E07_7F1E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6A4E_00E1,
        process_table_crc32: 0x8C3A_B4DB,
        video_crc32: 0xBF6F_8D99,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x77F8_C578,
        process_table_crc32: 0x8970_16F5,
        video_crc32: 0x5AE6_AA27,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAFDE_3DCB,
        process_table_crc32: 0x2F83_5339,
        video_crc32: 0x3529_74C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xABCB_E354,
        process_table_crc32: 0x3153_7E5D,
        video_crc32: 0x1765_E1C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA243_5CE8,
        process_table_crc32: 0xCAD2_CF33,
        video_crc32: 0x8B56_85BC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x754D_F26F,
        process_table_crc32: 0x0D21_241F,
        video_crc32: 0x9D91_6B66,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x608C_C606,
        process_table_crc32: 0x0339_AF87,
        video_crc32: 0xAB5F_842A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7D72_4D22,
        process_table_crc32: 0x04E0_0E53,
        video_crc32: 0xFF63_9059,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD871_6C24,
        process_table_crc32: 0x3C7A_6C24,
        video_crc32: 0xE3B5_F6AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC527_4011,
        process_table_crc32: 0x53DC_65AF,
        video_crc32: 0x593E_95A2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D01_B8A2,
        process_table_crc32: 0x61BF_DBA1,
        video_crc32: 0xB0DB_B848,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCA37_2C4E,
        process_table_crc32: 0x8286_C8CF,
        video_crc32: 0xF43D_8595,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x027C_33ED,
        process_table_crc32: 0xDD3D_75B9,
        video_crc32: 0x9F95_872A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFC3E_F257,
        process_table_crc32: 0x1602_74BB,
        video_crc32: 0x8407_5BC7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x99D0_EADF,
        process_table_crc32: 0x76BA_10BD,
        video_crc32: 0x9EC1_09A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x842E_61FB,
        process_table_crc32: 0x686A_3DD9,
        video_crc32: 0xF32E_91DF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE1E7_E357,
        process_table_crc32: 0x5580_D75B,
        video_crc32: 0x8767_1F86,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFC51_26CE,
        process_table_crc32: 0xC6AB_4C7B,
        video_crc32: 0x519A_48D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2477_DE7D,
        process_table_crc32: 0xEDC1_7EC5,
        video_crc32: 0xAF95_B4A7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBDA0_7340,
        process_table_crc32: 0xF311_53A1,
        video_crc32: 0x7185_ED86,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3555_F316,
        process_table_crc32: 0x8509_95BD,
        video_crc32: 0x0378_132C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE25B_5D91,
        process_table_crc32: 0xEA0C_98A7,
        video_crc32: 0xFADF_D8BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF79A_69F8,
        process_table_crc32: 0xFD1D_9F8F,
        video_crc32: 0xC6C4_F2EA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEA64_E2DC,
        process_table_crc32: 0x4B3B_54DD,
        video_crc32: 0xF4D4_B774,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1583_82CF,
        process_table_crc32: 0x5273_313C,
        video_crc32: 0xAA88_DC4D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0914_7DA2,
        process_table_crc32: 0x4E30_1FA2,
        video_crc32: 0xA796_1FBD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD132_8511,
        process_table_crc32: 0x2E64_812F,
        video_crc32: 0x0BF3_0607,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0604_11FD,
        process_table_crc32: 0x29BD_20FB,
        video_crc32: 0xC1CA_3D12,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCE4F_0E5E,
        process_table_crc32: 0x7606_9D8D,
        video_crc32: 0xA2F0_882A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x57A0_9908,
        process_table_crc32: 0x59D9_2E35,
        video_crc32: 0x7DAB_C340,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x02DF_3294,
        process_table_crc32: 0x57C1_A5AD,
        video_crc32: 0x7435_44AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1F21_B9B0,
        process_table_crc32: 0x5018_0479,
        video_crc32: 0xFA76_C6EB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7AE8_3B1C,
        process_table_crc32: 0xD8D3_6D37,
        video_crc32: 0x3B6E_6DCE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x675E_FE85,
        process_table_crc32: 0x92EB_2711,
        video_crc32: 0x158D_5A08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBF78_0636,
        process_table_crc32: 0xB981_15AF,
        video_crc32: 0xA0CE_A7D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5B98_39AC,
        process_table_crc32: 0x3130_01C5,
        video_crc32: 0x1F6F_CABF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8B9B_FF85,
        process_table_crc32: 0xCAB1_B0AB,
        video_crc32: 0xAE53_3286,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5C95_5102,
        process_table_crc32: 0xA5B4_BDB1,
        video_crc32: 0x6120_BD3B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4954_656B,
        process_table_crc32: 0x3F3C_CDEB,
        video_crc32: 0x8C3A_201F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x54AA_EE4F,
        process_table_crc32: 0x21EC_E08F,
        video_crc32: 0x78C5_3AC5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF1A9_CF49,
        process_table_crc32: 0x21AD_09DE,
        video_crc32: 0x8AB8_FC00,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xECFF_E37C,
        process_table_crc32: 0x8C11_4DC6,
        video_crc32: 0x0849_C296,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x34D9_1BCF,
        process_table_crc32: 0xBE72_F3C8,
        video_crc32: 0xBDFC_ABF2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE3EF_8F23,
        process_table_crc32: 0xB9AB_521C,
        video_crc32: 0x8DBD_A77F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2BA4_9080,
        process_table_crc32: 0x0914_4F41,
        video_crc32: 0x17F7_F869,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F89_7474,
        process_table_crc32: 0x7F18_CEEB,
        video_crc32: 0xD7A0_C70A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB8B_E002,
        process_table_crc32: 0x7100_4573,
        video_crc32: 0xBF5C_1933,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE675_6B26,
        process_table_crc32: 0x9239_561D,
        video_crc32: 0x7C5D_AD0C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x83BC_E98A,
        process_table_crc32: 0xB6DA_302F,
        video_crc32: 0x02D0_4B71,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9E0A_2C13,
        process_table_crc32: 0xAA99_1EB1,
        video_crc32: 0xDFE2_194A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x462C_D4A0,
        process_table_crc32: 0xF65A_4F21,
        video_crc32: 0xC579_BD18,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC05B_B8FE,
        process_table_crc32: 0xE88A_6245,
        video_crc32: 0xB557_6CC0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4E98_2994,
        process_table_crc32: 0x130B_D32B,
        video_crc32: 0x8CB2_085C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF1E_09DA,
        process_table_crc32: 0xEA6F_E73F,
        video_crc32: 0xEF58_C9FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8C6F_8911,
        process_table_crc32: 0xFD7E_E017,
        video_crc32: 0x4B68_6BD3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1AC6_7A4D,
        process_table_crc32: 0xE3AE_CD73,
        video_crc32: 0x1EED_E055,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4E53_6195,
        process_table_crc32: 0xDB34_AF04,
        video_crc32: 0xDE9B_FE73,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD8FA_92C9,
        process_table_crc32: 0x6B3C_F17E,
        video_crc32: 0xF354_B083,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8B8B_1202,
        process_table_crc32: 0x4056_C3C0,
        video_crc32: 0x707A_417B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1A0D_324C,
        process_table_crc32: 0xF670_0892,
        video_crc32: 0xD025_38A7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x94CE_A326,
        process_table_crc32: 0xA9CB_B5E4,
        video_crc32: 0xA2D8_6021,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0B17_254C,
        process_table_crc32: 0x62F4_B4E6,
        video_crc32: 0x40C6_977F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5866_A587,
        process_table_crc32: 0x3EDB_1FFD,
        video_crc32: 0xEABF_3439,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCECF_56DB,
        process_table_crc32: 0x3902_BE29,
        video_crc32: 0x4AB6_D0DF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2019_E2B2,
        process_table_crc32: 0x66B7_DFBD,
        video_crc32: 0xE7C9_FE7E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB6B0_11EE,
        process_table_crc32: 0x9E14_4399,
        video_crc32: 0x6169_26A5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE5C1_9125,
        process_table_crc32: 0xAC77_FD97,
        video_crc32: 0x87F7_14E9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x66A7_5B07,
        process_table_crc32: 0xABAE_5C43,
        video_crc32: 0x3DAC_6D3E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE864_CA6D,
        process_table_crc32: 0x2786_8E03,
        video_crc32: 0x7199_33A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x79E2_EA23,
        process_table_crc32: 0x4883_8319,
        video_crc32: 0x93C7_2A70,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2A93_6AE8,
        process_table_crc32: 0x5F92_8431,
        video_crc32: 0xE452_76E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBC3A_99B4,
        process_table_crc32: 0xD723_905B,
        video_crc32: 0xF552_DD7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x92C6_67DB,
        process_table_crc32: 0xD762_790A,
        video_crc32: 0x69EF_12D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x046F_9487,
        process_table_crc32: 0xD228_DB24,
        video_crc32: 0xE9AE_C9AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x571E_144C,
        process_table_crc32: 0x74DB_9EE8,
        video_crc32: 0x0BB1_B3F2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC698_3402,
        process_table_crc32: 0x6A0B_B38C,
        video_crc32: 0xA9EB_5EC0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x485B_A568,
        process_table_crc32: 0x2CB9_824A,
        video_crc32: 0xE567_F51F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF243_F7DA,
        process_table_crc32: 0x5679_E9CE,
        video_crc32: 0xC828_6095,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA132_7711,
        process_table_crc32: 0x5861_6256,
        video_crc32: 0x775F_3805,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x379B_844D,
        process_table_crc32: 0x5FB8_C382,
        video_crc32: 0xC037_1903,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD94D_3024,
        process_table_crc32: 0x5A89_A226,
        video_crc32: 0xE92E_2968,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4FE4_C378,
        process_table_crc32: 0x352F_ABAD,
        video_crc32: 0x6798_7D78,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C95_43B3,
        process_table_crc32: 0x074C_15A3,
        video_crc32: 0x9583_EC40,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x834C_C5D9,
        process_table_crc32: 0xE475_06CD,
        video_crc32: 0x0566_3096,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0D8F_54B3,
        process_table_crc32: 0x06FD_3B13,
        video_crc32: 0x7894_3D4E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9C09_74FD,
        process_table_crc32: 0x70F1_BAB9,
        video_crc32: 0x5527_39C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCF78_F436,
        process_table_crc32: 0x1049_DEBF,
        video_crc32: 0xB71D_7E45,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x59D1_076A,
        process_table_crc32: 0x0E99_F3DB,
        video_crc32: 0x3CDE_3903,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2C08_6B48,
        process_table_crc32: 0x0ED8_1A8A,
        video_crc32: 0x1F8D_EFB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBAA1_9814,
        process_table_crc32: 0x9DF3_81AA,
        video_crc32: 0x0A2F_AD35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE9D0_18DF,
        process_table_crc32: 0xB699_B314,
        video_crc32: 0x3F97_FDFF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7856_3891,
        process_table_crc32: 0xA849_9E70,
        video_crc32: 0x37D9_E3C3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF695_A9FB,
        process_table_crc32: 0xDE51_586C,
        video_crc32: 0x094B_B650,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x694C_2F91,
        process_table_crc32: 0xB154_5576,
        video_crc32: 0xFAA7_E4FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A3D_AF5A,
        process_table_crc32: 0xA645_525E,
        video_crc32: 0xA93E_8A04,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23F1_3F03,
        process_table_crc32: 0x799D_D04C,
        video_crc32: 0x03C7_9E4A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFADB_53E9,
        process_table_crc32: 0x4DB1_B402,
        video_crc32: 0xAA7E_2764,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD41B_F5C3,
        process_table_crc32: 0x1BF5_9BBE,
        video_crc32: 0xA752_4680,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF56C_0744,
        process_table_crc32: 0x31A6_0411,
        video_crc32: 0x6949_D03D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1EC6_57E4,
        process_table_crc32: 0xE876_A6A3,
        video_crc32: 0x0273_5DD1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFE64_2B3B,
        process_table_crc32: 0xD4F7_981B,
        video_crc32: 0xD70A_D4CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3505_A38D,
        process_table_crc32: 0x0C1C_AA29,
        video_crc32: 0x252D_3ED2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x20E4_0D41,
        process_table_crc32: 0x4803_2093,
        video_crc32: 0xFDBF_673E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8579_BC5B,
        process_table_crc32: 0x62BE_80E8,
        video_crc32: 0x85BA_C927,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8315_BE01,
        process_table_crc32: 0x7740_E330,
        video_crc32: 0x1C17_470D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF660_52C3,
        process_table_crc32: 0x777F_A834,
        video_crc32: 0x63E0_239C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEC21_AD99,
        process_table_crc32: 0x1612_9BA8,
        video_crc32: 0xCE3F_CA84,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x784A_BEE4,
        process_table_crc32: 0x40AA_8CA4,
        video_crc32: 0x5875_19A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x98E8_C23B,
        process_table_crc32: 0x6522_3EAC,
        video_crc32: 0x4E72_D77B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4564_B1C6,
        process_table_crc32: 0x4020_3294,
        video_crc32: 0xE787_461F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCCEA_DC7E,
        process_table_crc32: 0x35EA_27B8,
        video_crc32: 0x0015_9A94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6977_6D64,
        process_table_crc32: 0x9EB0_6CF4,
        video_crc32: 0x90FE_A6E8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x79F6_9475,
        process_table_crc32: 0xB395_840A,
        video_crc32: 0x59FF_AA2D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x90EC_BBC3,
        process_table_crc32: 0x542E_C130,
        video_crc32: 0x4864_F7E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB19B_4944,
        process_table_crc32: 0x2C4A_7E1C,
        video_crc32: 0xB047_71F9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A31_19E4,
        process_table_crc32: 0xF59A_DCAE,
        video_crc32: 0xD4F2_B1D3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBA93_653B,
        process_table_crc32: 0x156F_8B92,
        video_crc32: 0x3342_742A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x079E_F637,
        process_table_crc32: 0xA727_421D,
        video_crc32: 0xEAEE_6E64,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEDFD_DF41,
        process_table_crc32: 0xE338_C8A7,
        video_crc32: 0x0330_BA7C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4860_6E5B,
        process_table_crc32: 0x2D65_DA66,
        video_crc32: 0xD756_8072,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4E0C_6C01,
        process_table_crc32: 0x1949_BE28,
        video_crc32: 0x8DB6_DD76,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3B79_80C3,
        process_table_crc32: 0x4F0D_9194,
        video_crc32: 0x920F_F476,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0CE3_890F,
        process_table_crc32: 0xAF9A_4179,
        video_crc32: 0xCB06_155E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7B26_1ADB,
        process_table_crc32: 0x9910_EF24,
        video_crc32: 0xA947_52E7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9B84_6604,
        process_table_crc32: 0xBC98_5D2C,
        video_crc32: 0xC614_EAD8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4608_15F9,
        process_table_crc32: 0x0FFB_681A,
        video_crc32: 0xA4A8_F8CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCF86_7841,
        process_table_crc32: 0x52ED_6E10,
        video_crc32: 0xD62E_D9F2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6A1B_C95B,
        process_table_crc32: 0xD108_49E2,
        video_crc32: 0x65BF_D562,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x41AC_9466,
        process_table_crc32: 0xC4F6_2A3A,
        video_crc32: 0x0B6C_4144,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7AEE_991E,
        process_table_crc32: 0x3EF9_7562,
        video_crc32: 0xD567_8557,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x15AE_8A23,
        process_table_crc32: 0x5F94_46FE,
        video_crc32: 0x0C76_5141,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9728_CBE4,
        process_table_crc32: 0x37BB_8ECA,
        video_crc32: 0x9E33_D6FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5091_47E6,
        process_table_crc32: 0x0B3A_B072,
        video_crc32: 0x7FBF_F325,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x49A8_ED9E,
        process_table_crc32: 0x3731_30FA,
        video_crc32: 0x3BB2_4D4E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E11_619C,
        process_table_crc32: 0x22CF_5322,
        video_crc32: 0x75F3_099C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0C97_205B,
        process_table_crc32: 0x0BA4_3AB8,
        video_crc32: 0xAFD6_A6B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x63D7_3366,
        process_table_crc32: 0x3F88_5EF6,
        video_crc32: 0x342A_AF9F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5895_3E1E,
        process_table_crc32: 0x8D2C_C3F0,
        video_crc32: 0x0A22_6B5B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2FDC_A8E4,
        process_table_crc32: 0xF548_7CDC,
        video_crc32: 0x367D_C2E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAD5A_E923,
        process_table_crc32: 0x2C98_DE6E,
        video_crc32: 0x1B46_D5D9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6AE3_6521,
        process_table_crc32: 0x6342_4532,
        video_crc32: 0x8704_BCF8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF958_F766,
        process_table_crc32: 0xC38A_8FE0,
        video_crc32: 0x1655_1E89,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3EE1_7B64,
        process_table_crc32: 0x06AD_057A,
        video_crc32: 0xC288_AB6B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6D57_926E,
        process_table_crc32: 0x9ED3_136C,
        video_crc32: 0x96A6_75E0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8895_B96C,
        process_table_crc32: 0xB3F6_FB92,
        video_crc32: 0xADD0_E177,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB3D7_B414,
        process_table_crc32: 0xFCBB_589E,
        video_crc32: 0x6F94_8857,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDC97_A729,
        process_table_crc32: 0x104F_1C70,
        video_crc32: 0x7EE4_8278,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E11_E6EE,
        process_table_crc32: 0xD096_3272,
        video_crc32: 0x129B_F612,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x99A8_6AEC,
        process_table_crc32: 0xF51E_807A,
        video_crc32: 0x837A_8BC4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4EE4_B6AB,
        process_table_crc32: 0x78EA_6A74,
        video_crc32: 0x3D8B_CD2C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x895D_3AA9,
        process_table_crc32: 0x3CF5_E0CE,
        video_crc32: 0x9013_75A5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0BDB_7B6E,
        process_table_crc32: 0x1648_40B5,
        video_crc32: 0x74E6_140F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x649B_6853,
        process_table_crc32: 0x03B6_236D,
        video_crc32: 0x8721_9095,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5FD9_652B,
        process_table_crc32: 0x2617_2BC4,
        video_crc32: 0xFB9E_3BE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBA1B_4E29,
        process_table_crc32: 0x5E73_94E8,
        video_crc32: 0xA9CD_E8F6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x389D_0FEE,
        process_table_crc32: 0x6343_84E0,
        video_crc32: 0x6203_CE94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF24_83EC,
        process_table_crc32: 0xE2F1_3AF0,
        video_crc32: 0xBF70_E165,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6C9F_11AB,
        process_table_crc32: 0x63C9_3AD0,
        video_crc32: 0xC9A4_4EEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAB26_9DA9,
        process_table_crc32: 0x4976_5FF4,
        video_crc32: 0x6607_238E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x29A0_DC6E,
        process_table_crc32: 0x3C3F_774A,
        video_crc32: 0xFAA5_6577,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x458C_6B6C,
        process_table_crc32: 0x111A_9FB4,
        video_crc32: 0x9493_3DBF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7ECE_6614,
        process_table_crc32: 0xC836_05B6,
        video_crc32: 0x255F_F78E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x118E_7529,
        process_table_crc32: 0xA95B_362A,
        video_crc32: 0xBD42_8276,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9308_34EE,
        process_table_crc32: 0x6982_1828,
        video_crc32: 0xF4F0_1CFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x54B1_B8EC,
        process_table_crc32: 0xC193_DD52,
        video_crc32: 0xB04B_C8A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4D88_1294,
        process_table_crc32: 0xE491_D16A,
        video_crc32: 0x8E17_C8DB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8A31_9E96,
        process_table_crc32: 0xB987_D760,
        video_crc32: 0xA161_26EA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x08B7_DF51,
        process_table_crc32: 0x22C5_1D9D,
        video_crc32: 0xA161_26EA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x67F7_CC6C,
        process_table_crc32: 0x16E9_79D3,
        video_crc32: 0x5E7A_2928,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5CB5_C114,
        process_table_crc32: 0x40AD_566F,
        video_crc32: 0xA0F5_B096,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7702_9C29,
        process_table_crc32: 0x6AFE_C9C0,
        video_crc32: 0xF1A7_9921,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF584_DDEE,
        process_table_crc32: 0xB32E_6B72,
        video_crc32: 0x2093_4679,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x323D_51EC,
        process_table_crc32: 0x329C_D562,
        video_crc32: 0xF712_A5F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA186_C3AB,
        process_table_crc32: 0x5744_67F8,
        video_crc32: 0x2CD1_30B0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x663F_4FA9,
        process_table_crc32: 0x135B_ED42,
        video_crc32: 0x1EEB_4D6B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE4B9_0E6E,
        process_table_crc32: 0x044D_4EEA,
        video_crc32: 0xFDFD_3D29,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x017B_256C,
        process_table_crc32: 0x11B3_2D32,
        video_crc32: 0xA252_951C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A39_2814,
        process_table_crc32: 0x118C_6636,
        video_crc32: 0xB0E1_D21A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5579_3B29,
        process_table_crc32: 0x70E1_55AA,
        video_crc32: 0xEC52_D72E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD7FF_7AEE,
        process_table_crc32: 0x2659_42A6,
        video_crc32: 0x0C8A_0A57,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1046_F6EC,
        process_table_crc32: 0x03D1_F0AE,
        video_crc32: 0x0127_7038,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0FA6_14EA,
        process_table_crc32: 0x26D3_FC96,
        video_crc32: 0xEEB5_A99E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC81F_98E8,
        process_table_crc32: 0xF65C_8DEE,
        video_crc32: 0x6120_2270,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4A99_D92F,
        process_table_crc32: 0xC5E8_A125,
        video_crc32: 0xB7E9_7570,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x25D9_CA12,
        process_table_crc32: 0xE8CD_49DB,
        video_crc32: 0xB6F5_AB79,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1E9B_C76A,
        process_table_crc32: 0x0F76_0CE1,
        video_crc32: 0x9709_0D6F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB59_EC68,
        process_table_crc32: 0x7712_B3CD,
        video_crc32: 0x7F3E_872C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x79DF_ADAF,
        process_table_crc32: 0xAEC2_117F,
        video_crc32: 0x3E7E_D66C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBE66_21AD,
        process_table_crc32: 0xC074_0F44,
        video_crc32: 0xCB28_0A43,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2DDD_B3EA,
        process_table_crc32: 0xFC7F_8FCC,
        video_crc32: 0xC87A_B202,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEA64_3FE8,
        process_table_crc32: 0x1718_5672,
        video_crc32: 0xAFB2_9900,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x68E2_7E2F,
        process_table_crc32: 0x838E_2A0F,
        video_crc32: 0xD278_FDCB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4355_2312,
        process_table_crc32: 0x8564_AD93,
        video_crc32: 0x0990_2933,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7817_2E6A,
        process_table_crc32: 0x0EB8_BFDB,
        video_crc32: 0x9678_38FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1757_3D57,
        process_table_crc32: 0x6D6B_A4A1,
        video_crc32: 0x6A3B_871C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95D1_7C90,
        process_table_crc32: 0x2926_048B,
        video_crc32: 0x8092_96A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5268_F092,
        process_table_crc32: 0x2579_F49A,
        video_crc32: 0x95C4_67CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4B51_5AEA,
        process_table_crc32: 0x4B82_FC58,
        video_crc32: 0xD78F_A202,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8CE8_D6E8,
        process_table_crc32: 0x6963_22F6,
        video_crc32: 0x3426_94C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E6E_972F,
        process_table_crc32: 0x001C_6081,
        video_crc32: 0x4978_628A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x612E_8412,
        process_table_crc32: 0xA5CC_2A69,
        video_crc32: 0x804F_1C8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A6C_896A,
        process_table_crc32: 0x825B_48C5,
        video_crc32: 0x451A_1D8B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3640_3E68,
        process_table_crc32: 0x9621_3091,
        video_crc32: 0x5C3B_8C9F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB4C6_7FAF,
        process_table_crc32: 0x4CB9_351B,
        video_crc32: 0xCDE3_015D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x737F_F3AD,
        process_table_crc32: 0x59EF_49BA,
        video_crc32: 0x8710_0389,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE0C4_61EA,
        process_table_crc32: 0xB87C_F4C6,
        video_crc32: 0xBAEF_953E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x277D_EDE8,
        process_table_crc32: 0x4FF1_E4C8,
        video_crc32: 0xD870_3BC7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA5FB_AC2F,
        process_table_crc32: 0x3F87_2A0F,
        video_crc32: 0x6A98_373A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4039_872D,
        process_table_crc32: 0x396D_AD93,
        video_crc32: 0x0D12_1865,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7B7B_8A55,
        process_table_crc32: 0x5651_0D61,
        video_crc32: 0xC5F8_CF1A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x143B_9968,
        process_table_crc32: 0x5B22_F985,
        video_crc32: 0xB007_7F67,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x96BD_D8AF,
        process_table_crc32: 0x6A20_5233,
        video_crc32: 0xF4BB_EB4B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5104_54AD,
        process_table_crc32: 0x11D6_C10C,
        video_crc32: 0xC771_8948,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A56_30CD,
        process_table_crc32: 0x733F_F148,
        video_crc32: 0xA7F5_A5CA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x45B0_F298,
        video_crc32: 0x39DA_8CFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xCD01_E6F2,
        video_crc32: 0xCAD2_399A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xAA23_0F2E,
        video_crc32: 0x00B9_DCD9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xAF69_AD00,
        video_crc32: 0x371E_944F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x92A6_3E31,
        video_crc32: 0x471C_2A78,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x8344_C7EC,
        video_crc32: 0xBB22_8BC1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xECCB_74C6,
        video_crc32: 0xF5E3_EE95,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x2B38_9FEA,
        video_crc32: 0xA893_A2C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x4243_14FF,
        video_crc32: 0xAE5B_AC2B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x459A_B52B,
        video_crc32: 0xC0CB_D730,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xEC5B_245E,
        video_crc32: 0xEAA9_7256,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x6986_F0C4,
        video_crc32: 0x1C56_D413,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xD3A8_6210,
        video_crc32: 0xBE86_DBC2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x3091_717E,
        video_crc32: 0x28E8_DA25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x4617_4EE4,
        video_crc32: 0xA4E5_6948,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x301B_CF4E,
        video_crc32: 0xACB6_6B29,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xC312_5D65,
        video_crc32: 0x6CE3_1921,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x14BB_8572,
        video_crc32: 0xECBB_8564,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xB04F_8FDF,
        video_crc32: 0x93A2_ED25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xE0B2_F78E,
        video_crc32: 0x1CC6_A54C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x5FD6_C774,
        video_crc32: 0x2010_21C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x4106_EA10,
        video_crc32: 0xF5B3_C748,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xE8C7_7B65,
        video_crc32: 0x7406_B13B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x1E23_AEE0,
        video_crc32: 0x1B79_AA92,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xBC67_24F7,
        video_crc32: 0x7F3A_E804,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x0A41_EFA5,
        video_crc32: 0xAA39_2141,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x746A_8AC9,
        video_crc32: 0x1B19_280C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x6829_A457,
        video_crc32: 0xD759_FA89,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x55E6_3766,
        video_crc32: 0x04BD_6FC4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x9BAA_994A,
        video_crc32: 0x72FA_C4EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xED2C_A6D0,
        video_crc32: 0x96CF_AF82,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x9DCF_2D24,
        video_crc32: 0x461D_E261,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xB249_A244,
        video_crc32: 0xD1C7_93E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xB590_0390,
        video_crc32: 0x0896_FEC6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x1C51_92E5,
        video_crc32: 0x817A_91AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xA51B_8962,
        video_crc32: 0xA2BB_D821,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x1F35_1BB6,
        video_crc32: 0x0289_3FCF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x9784_0FDC,
        video_crc32: 0x8102_C136,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xF80B_BCF6,
        video_crc32: 0x15B8_073A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x970E_B1EC,
        video_crc32: 0x2366_16D3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x59AC_2014,
        video_crc32: 0x2A08_6A1D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x7435_EC5F,
        video_crc32: 0xD73C_20F6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x1317_0583,
        video_crc32: 0x1EDB_96E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xBEAB_419B,
        video_crc32: 0x8CD5_1208,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x18C6_FDD1,
        video_crc32: 0x3586_CCBD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0x1F1F_5C05,
        video_crc32: 0x4F12_D8CF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x977F_25B2,
        process_table_crc32: 0xB6DE_CD70,
        video_crc32: 0xA920_8B5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x4DA2_C2B6,
        video_crc32: 0x1E5C_8645,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x1972_4A70,
        video_crc32: 0x14FD_5605,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0xFA4B_591E,
        video_crc32: 0x3BF2_A148,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x8460_3C72,
        video_crc32: 0xB408_82AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x9823_12EC,
        video_crc32: 0xA5BF_3C07,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x50EE_4138,
        video_crc32: 0x6DE7_C3C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x4E3E_6C5C,
        video_crc32: 0x9094_2C4B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x21B1_DF76,
        video_crc32: 0xE9D5_F918,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0xD8D5_EB62,
        video_crc32: 0xCBFD_DFD8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0xA8A7_ECC7,
        video_crc32: 0x3F6C_99E0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0xB677_C1A3,
        video_crc32: 0xE61D_F45A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x1FB6_50D6,
        video_crc32: 0x6F12_4018,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x5986_FD23,
        video_crc32: 0xD284_3E62,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0xE6E2_CDD9,
        video_crc32: 0xA3BA_C28B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x50C4_068B,
        video_crc32: 0xDA56_4D57,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x464F_8D7F,
        process_table_crc32: 0x2642_3911,
        video_crc32: 0x3F02_B8EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB507_6802,
        process_table_crc32: 0xD2CC_8386,
        video_crc32: 0x2970_59A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD837_97AC,
        process_table_crc32: 0x310A_13D1,
        video_crc32: 0x55C9_9936,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE3CA_F0EF,
        process_table_crc32: 0x893A_8949,
        video_crc32: 0xF877_1DC8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD80D_2B89,
        process_table_crc32: 0x1853_8953,
        video_crc32: 0x6C0B_051A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC960_805B,
        process_table_crc32: 0x9BBC_7245,
        video_crc32: 0x1594_0EB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB04F_DD2E,
        process_table_crc32: 0xE3D8_CD69,
        video_crc32: 0x1B2E_F387,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1010_38F2,
        process_table_crc32: 0xAE06_6D9F,
        video_crc32: 0x3684_7AF8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8A1A_852F,
        process_table_crc32: 0x4DC0_FDC8,
        video_crc32: 0xD8F2_2A51,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5022_DABB,
        process_table_crc32: 0xFC27_BCB9,
        video_crc32: 0x9E1F_C4F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E4F_FBFE,
        process_table_crc32: 0xBE46_B00C,
        video_crc32: 0x8F65_C032,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0C83_6E34,
        process_table_crc32: 0xE350_B606,
        video_crc32: 0xAAF0_6C37,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE4C9_BDF2,
        process_table_crc32: 0x21E6_A34E,
        video_crc32: 0xA83C_6B43,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23CB_2577,
        process_table_crc32: 0x0CC3_4BB0,
        video_crc32: 0x2A6C_E29E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5AE4_7802,
        process_table_crc32: 0xD780_EAF8,
        video_crc32: 0x2194_E251,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB43F_65BE,
        process_table_crc32: 0x3446_7AAF,
        video_crc32: 0xEDC1_1241,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x63EF_36AE,
        process_table_crc32: 0x6FA3_8250,
        video_crc32: 0xF125_1F35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB9D7_693A,
        process_table_crc32: 0x4A2B_3058,
        video_crc32: 0x1F96_96FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA93E_B01F,
        process_table_crc32: 0x9D17_D908,
        video_crc32: 0xD340_D869,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE628_CB18,
        process_table_crc32: 0xD908_53B2,
        video_crc32: 0x946E_7969,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDDEF_107E,
        process_table_crc32: 0x94D6_F344,
        video_crc32: 0x0EC9_CD63,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCC82_BBAC,
        process_table_crc32: 0x7710_6313,
        video_crc32: 0x5032_4175,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB5AD_E6D9,
        process_table_crc32: 0xF2CD_B789,
        video_crc32: 0xBFCD_935D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6999_1404,
        process_table_crc32: 0x48E3_255D,
        video_crc32: 0xB52A_E5AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x843C_C936,
        process_table_crc32: 0xE1DD_3711,
        video_crc32: 0x21C0_F034,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E04_96A2,
        process_table_crc32: 0xDD5C_09A9,
        video_crc32: 0x4634_764E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0069_B7E7,
        process_table_crc32: 0x8634_89AC,
        video_crc32: 0x5C81_9AA0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x02A5_222D,
        process_table_crc32: 0x65F2_19FB,
        video_crc32: 0x1C58_5FB5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x77E6_012B,
        process_table_crc32: 0xF85C_C0CE,
        video_crc32: 0x782C_2AA1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2B51_4434,
        process_table_crc32: 0xD579_2830,
        video_crc32: 0x07D4_AA12,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x527E_1941,
        process_table_crc32: 0x985B_B076,
        video_crc32: 0x5BA0_1B9B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBCA5_04FD,
        process_table_crc32: 0xF936_83EA,
        video_crc32: 0xC561_2A2D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6B75_57ED,
        process_table_crc32: 0xADE1_AFAC,
        video_crc32: 0xAC6A_ECB0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB14D_0879,
        process_table_crc32: 0x4E27_3FFB,
        video_crc32: 0xEAA6_74DA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x443D_1CAE,
        process_table_crc32: 0xF7C0_6D5A,
        video_crc32: 0x3D20_0205,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8DA4_7D22,
        process_table_crc32: 0xAAD6_6B50,
        video_crc32: 0x0D68_58E8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7D36_5202,
        process_table_crc32: 0x56F7_A120,
        video_crc32: 0x5192_D3C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC2A4_F7B5,
        process_table_crc32: 0x62DB_C56E,
        video_crc32: 0xD748_6EAE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDE21_50E3,
        process_table_crc32: 0xA091_E896,
        video_crc32: 0x5A5E_9F57,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF8F1_AFB4,
        process_table_crc32: 0x4357_78C1,
        video_crc32: 0x8A95_EE6E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE474_08E2,
        process_table_crc32: 0xC71C_D7CF,
        video_crc32: 0x2F8B_B004,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5BE6_AD55,
        process_table_crc32: 0xFB9D_E977,
        video_crc32: 0xA2CD_123F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAB74_8275,
        process_table_crc32: 0x4415_DBC8,
        video_crc32: 0x1E57_2115,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x62ED_E3F9,
        process_table_crc32: 0x000A_5172,
        video_crc32: 0xE18F_B220,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDE36_A782,
        process_table_crc32: 0x4DD4_F184,
        video_crc32: 0x01B5_B758,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x61A4_0235,
        process_table_crc32: 0xAE12_61D3,
        video_crc32: 0x7BB1_06CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7D21_A563,
        process_table_crc32: 0x1758_7A54,
        video_crc32: 0x065A_1D5B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x58AF_4C99,
        process_table_crc32: 0xAD76_E880,
        video_crc32: 0xE4D3_94AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x442A_EBCF,
        process_table_crc32: 0x6FC0_FDC8,
        video_crc32: 0x0B77_506B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFBB8_4E78,
        process_table_crc32: 0x4A48_4FC0,
        video_crc32: 0x7473_2AC4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD3E5_7E24,
        process_table_crc32: 0x3582_40A6,
        video_crc32: 0xCF59_E2F1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1A7C_1FA8,
        process_table_crc32: 0xE50D_31DE,
        video_crc32: 0x8CFA_19AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEAEE_3088,
        process_table_crc32: 0xB1DA_1D98,
        video_crc32: 0x6DD0_139D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x557C_953F,
        process_table_crc32: 0x9CFF_F566,
        video_crc32: 0x2367_B580,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x49F9_3269,
        process_table_crc32: 0xEF4A_B218,
        video_crc32: 0x4EF3_2CE5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6995_E064,
        process_table_crc32: 0x972E_0D34,
        video_crc32: 0x3A2F_FE13,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7510_4732,
        process_table_crc32: 0xDAF0_ADC2,
        video_crc32: 0xF8E0_D353,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCA82_E285,
        process_table_crc32: 0x3936_3D95,
        video_crc32: 0x5921_FA40,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A10_CDA5,
        process_table_crc32: 0x0975_3351,
        video_crc32: 0xFEC0_B9D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF389_AC29,
        process_table_crc32: 0xAB31_B946,
        video_crc32: 0xE8D8_9A02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0045_95A4,
        process_table_crc32: 0x020F_AB0A,
        video_crc32: 0xC287_84C4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBFD7_3013,
        process_table_crc32: 0x3623_CF44,
        video_crc32: 0x42B7_2640,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA352_9745,
        process_table_crc32: 0xF469_E2BC,
        video_crc32: 0x28D3_1DB7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x86DC_7EBF,
        process_table_crc32: 0xE2AD_B20E,
        video_crc32: 0x7E48_4940,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9A59_D9E9,
        process_table_crc32: 0xB67A_9E48,
        video_crc32: 0x38ED_98A2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x25CB_7C5E,
        process_table_crc32: 0x93F2_2C40,
        video_crc32: 0x765A_3EBF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDDC3_323D,
        process_table_crc32: 0x010F_1D8E,
        video_crc32: 0xE050_0FF0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x145A_53B1,
        process_table_crc32: 0x5C19_1B84,
        video_crc32: 0x6553_55E9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE4C8_7C91,
        process_table_crc32: 0x08CE_37C2,
        video_crc32: 0x0C37_4E9B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5B5A_D926,
        process_table_crc32: 0xEB08_A795,
        video_crc32: 0x85BD_B5C3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x47DF_7E70,
        process_table_crc32: 0x7331_0906,
        video_crc32: 0xC6A6_652E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x610F_8127,
        process_table_crc32: 0x125C_3A9A,
        video_crc32: 0x62CF_CB66,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7D8A_2671,
        process_table_crc32: 0xEE7D_F0EA,
        video_crc32: 0x3E29_3541,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC218_83C6,
        process_table_crc32: 0xD2FC_CE52,
        video_crc32: 0x3E29_3541,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x328A_ACE6,
        process_table_crc32: 0x8994_4E57,
        video_crc32: 0x6E42_67E9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB13_CD6A,
        process_table_crc32: 0x9FBC_E46E,
        video_crc32: 0x2F06_4C03,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E63_D9BD,
        process_table_crc32: 0xD262_4498,
        video_crc32: 0x99E7_AE55,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB1F1_7C0A,
        process_table_crc32: 0xE64E_20D6,
        video_crc32: 0xCC55_4A42,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAD74_DB5C,
        process_table_crc32: 0xC0E4_BF94,
        video_crc32: 0x6737_ECE4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x88FA_32A6,
        process_table_crc32: 0xB880_00B8,
        video_crc32: 0x1CBB_200C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x947F_95F0,
        process_table_crc32: 0xF55E_A04E,
        video_crc32: 0xBCAB_221B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2BED_3047,
        process_table_crc32: 0x1698_3019,
        video_crc32: 0xFB76_60FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD821_09CA,
        process_table_crc32: 0xD8B5_7E0E,
        video_crc32: 0xC6C6_670F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x11B8_6846,
        process_table_crc32: 0x85A3_7804,
        video_crc32: 0x2CFA_B02D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE12A_4766,
        process_table_crc32: 0x4715_6D4C,
        video_crc32: 0xE0EF_6E37,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EB8_E2D1,
        process_table_crc32: 0x6A30_85B2,
        video_crc32: 0x9269_E3B0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x423D_4587,
        process_table_crc32: 0xB173_24FA,
        video_crc32: 0x9622_2D8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x75D9_7856,
        process_table_crc32: 0x5D87_6014,
        video_crc32: 0xAAD3_C226,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x695C_DF00,
        process_table_crc32: 0x0950_4C52,
        video_crc32: 0x0EED_2B17,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD6CE_7AB7,
        process_table_crc32: 0x2CD8_FE5A,
        video_crc32: 0xDD35_C130,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x265C_5597,
        process_table_crc32: 0xC64F_14D9,
        video_crc32: 0x6DA3_65FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEFC5_341B,
        process_table_crc32: 0x8250_9E63,
        video_crc32: 0xB34E_7C3C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C09_0D96,
        process_table_crc32: 0xCF8E_3E95,
        video_crc32: 0xFC35_142C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA39B_A821,
        process_table_crc32: 0x2C48_AEC2,
        video_crc32: 0x9294_9F11,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBF1E_0F77,
        process_table_crc32: 0x6BDF_57A0,
        video_crc32: 0x0AE6_7767,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9A90_E68D,
        process_table_crc32: 0x13BB_E88C,
        video_crc32: 0x9165_DC79,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8615_41DB,
        process_table_crc32: 0xBA85_FAC0,
        video_crc32: 0xBDD0_DBD0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3987_E46C,
        process_table_crc32: 0x9F72_2F8C,
        video_crc32: 0xD7DD_3B55,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCCF7_F0BB,
        process_table_crc32: 0x9ED1_C135,
        video_crc32: 0x7911_D001,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x056E_9137,
        process_table_crc32: 0x86A8_47C3,
        video_crc32: 0x5840_F3C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF5FC_BE17,
        process_table_crc32: 0x0FE7_5671,
        video_crc32: 0x4013_3095,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4A6E_1BA0,
        process_table_crc32: 0x57D5_F547,
        video_crc32: 0x55A7_58C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x56EB_BCF6,
        process_table_crc32: 0x9E63_E329,
        video_crc32: 0xA9A1_E859,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x703B_43A1,
        process_table_crc32: 0xD6D9_92AC,
        video_crc32: 0x88AC_AAB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6CBE_E4F7,
        process_table_crc32: 0x5F96_831E,
        video_crc32: 0x8849_F62A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD32C_4140,
        process_table_crc32: 0xC3A7_CBED,
        video_crc32: 0xC9F2_E94C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23BE_6E60,
        process_table_crc32: 0x3543_1E68,
        video_crc32: 0x9127_D706,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEA27_0FEC,
        process_table_crc32: 0x7DE2_D3F3,
        video_crc32: 0xC87D_5C82,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x122F_418F,
        process_table_crc32: 0x5C5B_2477,
        video_crc32: 0x93B5_E473,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xADBD_E438,
        process_table_crc32: 0x1D60_0BF1,
        video_crc32: 0x2C5D_9F33,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB138_436E,
        process_table_crc32: 0x6D9D_EBB7,
        video_crc32: 0xF2D5_EA87,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x94B6_AA94,
        process_table_crc32: 0x6E19_3601,
        video_crc32: 0x8DC0_C1DA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8833_0DC2,
        process_table_crc32: 0xFE5F_AB03,
        video_crc32: 0xF81A_9C27,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x37A1_A875,
        process_table_crc32: 0x237B_2F8C,
        video_crc32: 0x020A_C0E3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC46D_91F8,
        process_table_crc32: 0xC638_738F,
        video_crc32: 0xB648_745C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0DF4_F074,
        process_table_crc32: 0xB0E1_1AE7,
        video_crc32: 0xC00D_0947,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFD66_DF54,
        process_table_crc32: 0x20A7_87E5,
        video_crc32: 0xB2CF_B900,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x42F4_7AE3,
        process_table_crc32: 0xB676_5C7A,
        video_crc32: 0xC0C1_8BE6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E71_DDB5,
        process_table_crc32: 0x3CAD_EFB1,
        video_crc32: 0x0796_C932,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7E1D_0FB8,
        process_table_crc32: 0x7417_9E34,
        video_crc32: 0x13BE_7DAC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE725_579D,
        process_table_crc32: 0xF14A_B700,
        video_crc32: 0xE0BD_7950,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xBF5B_00D2,
        video_crc32: 0xBEEF_7094,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xD05E_0DC8,
        video_crc32: 0xF5F7_990F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x799F_9CBD,
        video_crc32: 0x1D9F_FFF1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x5406_50F6,
        video_crc32: 0xA53B_1170,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x64FC_581F,
        video_crc32: 0xD6BD_3312,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x0A96_FF76,
        video_crc32: 0xAFCE_E94F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x38F5_4178,
        video_crc32: 0x82B8_D48F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x3F2C_E0AC,
        video_crc32: 0x32B9_3E3A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xF18E_7154,
        video_crc32: 0x842C_59C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x7CFE_FF38,
        video_crc32: 0x6949_3144,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x04EA_F50A,
        video_crc32: 0x95C6_F7FB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xE7D3_E664,
        video_crc32: 0xC4C0_C1E0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xD729_EE8D,
        video_crc32: 0xFFC7_92FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x11B5_AFD2,
        video_crc32: 0x8EB3_AB6A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xB874_3EA7,
        video_crc32: 0x1EAA_66C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x53A6_D326,
        video_crc32: 0xC074_603B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x66E1_6352,
        video_crc32: 0x2240_713F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x9F85_5746,
        video_crc32: 0x1F50_DADB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x8894_506E,
        video_crc32: 0x97DA_CCE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x9644_7D0A,
        video_crc32: 0x62AD_439E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xAB8B_EE3B,
        video_crc32: 0x9E6B_B444,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xE8F1_E1E0,
        video_crc32: 0xD867_05C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xC6D1_7170,
        video_crc32: 0x5BB2_8AD6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x70F7_BA22,
        video_crc32: 0x50AA_07FB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x6112_8535,
        video_crc32: 0xE9B8_5E00,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x171E_049F,
        video_crc32: 0x8E16_541E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xBEDF_95EA,
        video_crc32: 0xA413_786D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x4CE8_0E50,
        video_crc32: 0x1EA9_A9D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x7C12_06B9,
        video_crc32: 0x6718_24C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x5E6E_F55C,
        video_crc32: 0x429C_0AA4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x6C0D_4B52,
        video_crc32: 0x44F4_6FC8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x89DB_5262,
        video_crc32: 0xDD63_C432,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x0184_C7EF,
        video_crc32: 0x6412_53B7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xB063_869E,
        video_crc32: 0xF068_7854,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xDF66_8B84,
        video_crc32: 0x9647_61AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xC877_8CAC,
        video_crc32: 0x5BB1_80E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x40C6_98C6,
        video_crc32: 0x031D_2A71,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x703C_902F,
        video_crc32: 0x3763_3420,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xB6A0_D170,
        video_crc32: 0x7F01_6F02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x1F61_4005,
        video_crc32: 0xB1B4_64FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xB230_8F4E,
        video_crc32: 0x4B08_0153,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x066F_0A7F,
        video_crc32: 0xA263_2264,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xC19C_E153,
        video_crc32: 0x1A36_663F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xCF84_6ACB,
        video_crc32: 0xB8BF_10E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xC85D_CB1F,
        video_crc32: 0x216D_D31E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0xF592_582E,
        video_crc32: 0x7FFF_697B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x704F_8CB4,
        video_crc32: 0xFC24_2541,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x857F_CABB,
        process_table_crc32: 0x5E6F_1C24,
        video_crc32: 0xE028_525D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xBD56_0F4A,
        video_crc32: 0x0FCC_7C2F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x9118_338E,
        video_crc32: 0x127B_74C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xE714_B224,
        video_crc32: 0xBFF8_DE69,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x4ED5_2351,
        video_crc32: 0x3028_2660,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x997C_FB46,
        video_crc32: 0xC863_CBEF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x6A50_10DE,
        video_crc32: 0xDC52_439F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xF97B_8BFE,
        video_crc32: 0xA074_9ECB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xD211_B940,
        video_crc32: 0xF5EB_0A9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xCCC1_9424,
        video_crc32: 0xB0D0_36BF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x0263_05DC,
        video_crc32: 0x4995_5D81,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x26B1_5DEB,
        video_crc32: 0xB904_D754,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x31A0_5AC3,
        video_crc32: 0xB81B_5E15,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x8786_9191,
        video_crc32: 0xF416_359F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x6DA3_F6B9,
        video_crc32: 0x3935_A977,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0x71E0_D827,
        video_crc32: 0xDFA9_B819,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EEE_C36A,
        process_table_crc32: 0xD821_4952,
        video_crc32: 0x1906_81CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC2B2_54C0,
        process_table_crc32: 0x94EF_DC43,
        video_crc32: 0xB47A_207F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF8D5_8EA7,
        process_table_crc32: 0xA86E_E2FB,
        video_crc32: 0xA1D6_01CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5859_4C01,
        process_table_crc32: 0x7085_D0C9,
        video_crc32: 0xAA19_82D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E2B_1356,
        process_table_crc32: 0x349A_5A73,
        video_crc32: 0x7463_8A61,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB1C4_6406,
        process_table_crc32: 0xED4A_F8C1,
        video_crc32: 0x26A5_24FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9D5D_C61F,
        process_table_crc32: 0x0E8C_6896,
        video_crc32: 0x34AC_E158,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF9EC_3DCA,
        process_table_crc32: 0xB7C6_7311,
        video_crc32: 0xDAF6_6A5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x09B5_FF4B,
        process_table_crc32: 0x99E6_E381,
        video_crc32: 0x68D7_09A0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAA5C_CBE6,
        process_table_crc32: 0x3C33_F644,
        video_crc32: 0x00EB_15AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF95_366F,
        process_table_crc32: 0x19BB_444C,
        video_crc32: 0xDF97_300E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCCD2_6EF7,
        process_table_crc32: 0x3CB9_4874,
        video_crc32: 0xE559_3A17,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1630_F681,
        process_table_crc32: 0xDF7F_D823,
        video_crc32: 0xB022_9F0F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD728_055D,
        process_table_crc32: 0x2CEF_170E,
        video_crc32: 0xD855_A736,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFBB1_A744,
        process_table_crc32: 0x01CA_FFF0,
        video_crc32: 0xF712_3E1D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2DA0_E3B1,
        process_table_crc32: 0xE671_BACA,
        video_crc32: 0x8B2B_6BF8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3E84_A08B,
        process_table_crc32: 0x9E15_05E6,
        video_crc32: 0x1658_B9AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9D6D_9426,
        process_table_crc32: 0x8903_A64E,
        video_crc32: 0x55CB_7C05,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A04_D68F,
        process_table_crc32: 0x6AC5_3619,
        video_crc32: 0xBBEC_D479,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAA3E_0FAC,
        process_table_crc32: 0x5A86_38DD,
        video_crc32: 0x9D57_BC19,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7C4C_50FB,
        process_table_crc32: 0x9FA1_B247,
        video_crc32: 0xA13F_1AD1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x43A3_27AB,
        process_table_crc32: 0xA291_A24F,
        video_crc32: 0xA2DB_BE45,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6F3A_85B2,
        process_table_crc32: 0x96BD_C601,
        video_crc32: 0xBE23_AA4F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x12AA_F025,
        process_table_crc32: 0xC0F9_E9BD,
        video_crc32: 0x2B2B_C51F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC46D_3DFD,
        process_table_crc32: 0x233F_79EA,
        video_crc32: 0x42D8_D925,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6784_0950,
        process_table_crc32: 0xE589_95C4,
        video_crc32: 0x9DAD_F96E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x124D_F4D9,
        process_table_crc32: 0xC001_27CC,
        video_crc32: 0x008F_88FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x010A_AC41,
        process_table_crc32: 0x7362_12FA,
        video_crc32: 0x0882_1918,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65D8_4C36,
        process_table_crc32: 0x2E74_14F0,
        video_crc32: 0xA058_31C3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB94A_BADD,
        process_table_crc32: 0xEEAD_3AF2,
        video_crc32: 0xC0F8_9CAE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95D3_18C4,
        process_table_crc32: 0x0D6B_AAA5,
        video_crc32: 0xA63C_3E32,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x43C2_5C31,
        process_table_crc32: 0x4E11_A57E,
        video_crc32: 0x2D9A_56F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x50E6_1F0B,
        process_table_crc32: 0x6031_35EE,
        video_crc32: 0x3E8A_36A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF30F_2BA6,
        process_table_crc32: 0x4B22_F42A,
        video_crc32: 0xBBC3_322F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4B1C_74A1,
        process_table_crc32: 0x77A3_CA92,
        video_crc32: 0xC6F7_945F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC45D_6FC4,
        process_table_crc32: 0x4BA8_4A1A,
        video_crc32: 0x23B9_2E98,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E29_736E,
        process_table_crc32: 0xA86E_DA4D,
        video_crc32: 0x6A1D_31F0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8E7B_E51D,
        process_table_crc32: 0x8450_4291,
        video_crc32: 0x9346_AA23,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0159_E5DA,
        process_table_crc32: 0xB07C_26DF,
        video_crc32: 0x3B0C_FE68,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1A93_DC49,
        process_table_crc32: 0x02D8_BBD9,
        video_crc32: 0x0550_AAEF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95B1_DC8E,
        process_table_crc32: 0x7ABC_04F5,
        video_crc32: 0xAC10_ECCB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95E3_4AFD,
        process_table_crc32: 0x5001_A48E,
        video_crc32: 0x78A5_934B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF97_5657,
        process_table_crc32: 0xB3C7_34D9,
        video_crc32: 0xDF2B_73F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x50D6_4D32,
        process_table_crc32: 0x2722_7990,
        video_crc32: 0x1EFA_8E38,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB137_1E9A,
        process_table_crc32: 0x7A34_7F9A,
        video_crc32: 0x3EC0_9ABF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB165_88E9,
        process_table_crc32: 0x2C8C_6896,
        video_crc32: 0x6939_E192,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3E47_882E,
        process_table_crc32: 0x01A9_8068,
        video_crc32: 0x41DA_8776,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7450_8F26,
        process_table_crc32: 0x4EE4_2364,
        video_crc32: 0x197F_5C9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB72_8FE1,
        process_table_crc32: 0xAD22_B333,
        video_crc32: 0xFC28_B936,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC85B_82C4,
        process_table_crc32: 0xAC0F_4892,
        video_crc32: 0xC7CD_FBBF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD3F2_A0F5,
        process_table_crc32: 0x8987_FA9A,
        video_crc32: 0x07BC_8511,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5CB3_BB90,
        process_table_crc32: 0x0473_1094,
        video_crc32: 0x93B5_812E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x16C7_A73A,
        process_table_crc32: 0x406C_9A2E,
        video_crc32: 0x0057_861B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1695_3149,
        process_table_crc32: 0x99BC_389C,
        video_crc32: 0x6330_7B45,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x99B7_318E,
        process_table_crc32: 0x8C42_5B44,
        video_crc32: 0x036A_CBE4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x21C7_752B,
        process_table_crc32: 0xA9E3_53ED,
        video_crc32: 0xA75B_E272,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAEE5_75EC,
        process_table_crc32: 0xD187_ECC1,
        video_crc32: 0xF2DD_86EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAEB7_E39F,
        process_table_crc32: 0x1FDA_FE00,
        video_crc32: 0x15E3_0DF2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE4C3_FF35,
        process_table_crc32: 0x235B_C0B8,
        video_crc32: 0x79FB_059B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6B82_E450,
        process_table_crc32: 0x1F50_4030,
        video_crc32: 0x1FA9_297B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x702B_C661,
        process_table_crc32: 0xFC96_D067,
        video_crc32: 0x7055_3899,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7079_5012,
        process_table_crc32: 0xF536_0B16,
        video_crc32: 0xCDBC_0719,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF5B_50D5,
        process_table_crc32: 0xD813_E3E8,
        video_crc32: 0xC6A2_37B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB54C_57DD,
        process_table_crc32: 0x013F_79EA,
        video_crc32: 0xEA86_C4AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A6E_571A,
        process_table_crc32: 0x6052_4A76,
        video_crc32: 0x2819_AD9E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3A3C_C169,
        process_table_crc32: 0x151B_62C8,
        video_crc32: 0xBEA5_59B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1E2A_6243,
        process_table_crc32: 0xF6DD_F29F,
        video_crc32: 0x162F_D9DE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x916B_7926,
        process_table_crc32: 0xBD0A_A7B2,
        video_crc32: 0xB694_F3C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDB1F_658C,
        process_table_crc32: 0xC51E_AD80,
        video_crc32: 0x4DBD_B47C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDB4D_F3FF,
        process_table_crc32: 0xAD31_65B4,
        video_crc32: 0x81B3_4EA4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x546F_F338,
        process_table_crc32: 0x991D_01FA,
        video_crc32: 0xF15C_845F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4FA5_CAAB,
        process_table_crc32: 0xCF59_2E46,
        video_crc32: 0x25F3_14B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC087_CA6C,
        process_table_crc32: 0xE50A_B1E9,
        video_crc32: 0xD3A7_B17C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC0D5_5C1F,
        process_table_crc32: 0xCFB7_1192,
        video_crc32: 0x595D_CC64,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8AA1_40B5,
        process_table_crc32: 0xF336_2F2A,
        video_crc32: 0x7EEC_980D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x05E0_5BD0,
        process_table_crc32: 0x2BDD_1D18,
        video_crc32: 0x746E_3811,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBDF3_04D7,
        process_table_crc32: 0x6FC2_97A2,
        video_crc32: 0xF7D8_7AF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBDA1_92A4,
        process_table_crc32: 0xB612_3510,
        video_crc32: 0xE55B_CC94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3283_9263,
        process_table_crc32: 0x55D4_A547,
        video_crc32: 0xE90E_98B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7894_956B,
        process_table_crc32: 0xA3D3_1DCC,
        video_crc32: 0x4034_63CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF7B6_95AC,
        process_table_crc32: 0xC2BE_2E50,
        video_crc32: 0xD0C7_DD6F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF7E4_03DF,
        process_table_crc32: 0x5AC0_3846,
        video_crc32: 0x4DA0_F9FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEC4D_21EE,
        process_table_crc32: 0x7F48_8A4E,
        video_crc32: 0x60F4_9AF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x630C_3A8B,
        process_table_crc32: 0x5A4A_8676,
        video_crc32: 0x8212_2E94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2978_2621,
        process_table_crc32: 0xB98C_1621,
        video_crc32: 0xE872_5914,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x292A_B052,
        process_table_crc32: 0x4A1C_D90C,
        video_crc32: 0xE872_5914,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA608_B095,
        process_table_crc32: 0x6739_31F2,
        video_crc32: 0x580D_34A7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6107_F606,
        process_table_crc32: 0x8082_74C8,
        video_crc32: 0x9923_79A9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEE25_F6C1,
        process_table_crc32: 0xF8E6_CBE4,
        video_crc32: 0x703D_AA7C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEE77_60B2,
        process_table_crc32: 0xD25B_6B9F,
        video_crc32: 0x72B1_64B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA403_7C18,
        process_table_crc32: 0xBCED_75A4,
        video_crc32: 0x5433_D423,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2B42_677D,
        process_table_crc32: 0x80E6_F52C,
        video_crc32: 0xFC56_810E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x30EB_454C,
        process_table_crc32: 0xC4F9_7F96,
        video_crc32: 0x8C8E_1A88,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x30B9_D33F,
        process_table_crc32: 0xF9C9_6F9E,
        video_crc32: 0x1C1D_0AAE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBF9B_D3F8,
        process_table_crc32: 0xCDE5_0BD0,
        video_crc32: 0x0573_3513,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF58C_D4F0,
        process_table_crc32: 0x9BA1_246C,
        video_crc32: 0x07C7_27E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7AAE_D437,
        process_table_crc32: 0x6111_5FCF,
        video_crc32: 0x1434_ECEA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7AFC_4244,
        process_table_crc32: 0xFD6C_DD5D,
        video_crc32: 0x9D43_246B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC2EF_1D43,
        process_table_crc32: 0xEA22_8C87,
        video_crc32: 0x09CE_531E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DAE_0626,
        process_table_crc32: 0x84D9_8445,
        video_crc32: 0x5B9A_D30A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x07DA_1A8C,
        process_table_crc32: 0xACD8_C987,
        video_crc32: 0x800E_1EA6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0788_8CFF,
        process_table_crc32: 0xE895_69AD,
        video_crc32: 0xD45E_00B4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x88AA_8C38,
        process_table_crc32: 0x2284_BBE3,
        video_crc32: 0xEF1F_A602,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9360_B5AB,
        process_table_crc32: 0xF32B_2AC0,
        video_crc32: 0xE455_6B25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C42_B56C,
        process_table_crc32: 0xEDB1_C1F8,
        video_crc32: 0xDE71_FA07,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C10_231F,
        process_table_crc32: 0x2C38_65B9,
        video_crc32: 0x1820_5A35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5664_3FB5,
        process_table_crc32: 0xA097_7231,
        video_crc32: 0x5917_E5E2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD925_24D0,
        process_table_crc32: 0x4104_CF4D,
        video_crc32: 0x3FF6_20F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFD33_87FA,
        process_table_crc32: 0x21ED_E75D,
        video_crc32: 0x74DC_F9D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFD61_1189,
        process_table_crc32: 0x495C_41B0,
        video_crc32: 0x1EF9_61FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7243_114E,
        process_table_crc32: 0x54A7_67E7,
        video_crc32: 0xC076_89D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3854_1646,
        process_table_crc32: 0x3B9B_C715,
        video_crc32: 0x0D5A_2078,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB776_1681,
        process_table_crc32: 0xA25A_C20E,
        video_crc32: 0xFC27_CB0E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB724_80F2,
        process_table_crc32: 0xD22C_0CC9,
        video_crc32: 0x086B_D01A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAC8D_A2C3,
        process_table_crc32: 0x032C_7F4C,
        video_crc32: 0x5134_6CF5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23CC_B9A6,
        process_table_crc32: 0xB2CB_3E3D,
        video_crc32: 0x9698_320F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x69B8_A50C,
        process_table_crc32: 0x4A51_0FF1,
        video_crc32: 0x5164_1FB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x69EA_337F,
        process_table_crc32: 0x6250_4233,
        video_crc32: 0x0B9C_A047,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE6C8_33B8,
        process_table_crc32: 0xDC3A_5C3B,
        video_crc32: 0xD57B_2864,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5EB8_771D,
        process_table_crc32: 0xD8C8_F6DC,
        video_crc32: 0xD2EC_4FCA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1607_AD79,
        process_table_crc32: 0xD06E_69AC,
        video_crc32: 0xD17B_D95E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x5831_FC21,
        video_crc32: 0xF791_766F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x49D3_05FC,
        video_crc32: 0xF7F2_914F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x265C_B6D6,
        video_crc32: 0xF82D_D7CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xE1AF_5DFA,
        video_crc32: 0x73A8_B2F9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x7BB9_D426,
        video_crc32: 0xDA73_3D6C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x7C60_75F2,
        video_crc32: 0xFBDB_EF33,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xD5A1_E487,
        video_crc32: 0x6959_E15D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x507C_301D,
        video_crc32: 0x9B16_BC56,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x193F_A000,
        video_crc32: 0xC34C_1E32,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xFA06_B36E,
        video_crc32: 0x8E9D_44A3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x8C80_8CF4,
        video_crc32: 0xB25D_2FCA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xFA8C_0D5E,
        video_crc32: 0xCD3C_4863,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xC743_9E6F,
        video_crc32: 0x28CE_9EA2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x0E3A_6B1C,
        video_crc32: 0x83F9_9CE6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xB41E_4CD5,
        video_crc32: 0xEA58_02C4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xE4E3_3484,
        video_crc32: 0xD034_003C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x9541_0564,
        video_crc32: 0xDE84_E7D3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x8B91_2800,
        video_crc32: 0xFB24_7111,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x2250_B975,
        video_crc32: 0xF38B_B120,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x6987_EC58,
        video_crc32: 0x244C_658A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x859D_E42E,
        video_crc32: 0x73F8_003D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x33BB_2F7C,
        video_crc32: 0x1FB2_4B3B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x974F_25D1,
        video_crc32: 0x073B_F578,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x51D3_648E,
        video_crc32: 0x5F01_40AC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x9F71_F576,
        video_crc32: 0xF188_D367,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x513D_5B5A,
        video_crc32: 0xB7A6_28F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x27BB_64C0,
        video_crc32: 0xDE2C_9263,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xB557_57D0,
        video_crc32: 0xE00A_689E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x2F41_DE0C,
        video_crc32: 0x7BBF_F5FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x2898_7FD8,
        video_crc32: 0xD28A_E9F1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x8159_EEAD,
        video_crc32: 0xC2C2_0186,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xBCC1_FEC1,
        video_crc32: 0x3C36_4777,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xDF56_EFE0,
        video_crc32: 0xC40F_9369,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x5D13_CDCC,
        video_crc32: 0xF17A_1C1C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x329C_7EE6,
        video_crc32: 0xC92A_C6F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x5D99_73FC,
        video_crc32: 0xCEEE_00BE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x6056_E0CD,
        video_crc32: 0x6E82_F684,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x531F_01E2,
        video_crc32: 0x95AF_AFE2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xE93B_262B,
        video_crc32: 0x70AC_2672,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x8751_8142,
        video_crc32: 0x9A43_E5B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xD251_3FC1,
        video_crc32: 0x9334_1762,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xD588_9E15,
        video_crc32: 0xC463_913B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x7C49_0F60,
        video_crc32: 0xAB54_C37C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xF139_810C,
        video_crc32: 0x39C4_ABE6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x8735_00A6,
        video_crc32: 0x13EF_8D47,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x1D23_897A,
        video_crc32: 0xE743_654F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0xFE1A_9A14,
        video_crc32: 0xD186_CBB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5FC_91FB,
        process_table_crc32: 0x5AEE_90B9,
        video_crc32: 0x102A_6434,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x9C72_D1E6,
        video_crc32: 0x69C2_77FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x6F7B_43CD,
        video_crc32: 0x992F_DF3D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x84A9_AE4C,
        video_crc32: 0x91F4_84D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0xEB26_1D66,
        video_crc32: 0x063F_E8E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x1242_2972,
        video_crc32: 0x2F2D_C182,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x915D_2C1E,
        video_crc32: 0x7817_6989,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x8F8D_017A,
        video_crc32: 0xD49A_B7D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x264C_900F,
        video_crc32: 0x9AE5_BEF5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x6536_9FD4,
        video_crc32: 0x947B_1C89,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x2C75_0FC9,
        video_crc32: 0x5FCF_7C5E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x9A53_C49B,
        video_crc32: 0xE7BA_D9E5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0xECD5_FB01,
        video_crc32: 0x5BFC_6E7A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x9AD9_7AAB,
        video_crc32: 0xDD22_E82B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0xA716_E99A,
        video_crc32: 0xDF85_5E0A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x5521_7220,
        video_crc32: 0x1DA7_5603,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF687_0AAD,
        process_table_crc32: 0x2B0A_174C,
        video_crc32: 0x7292_3394,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3965_78AC,
        process_table_crc32: 0x512B_B055,
        video_crc32: 0xDED7_B266,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x95EA_0E07,
        process_table_crc32: 0x294F_0F79,
        video_crc32: 0x73DC_1166,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6F2D_AB70,
        process_table_crc32: 0x6491_AF8F,
        video_crc32: 0x8D79_5A77,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x80FE_3C71,
        process_table_crc32: 0x8757_3FD8,
        video_crc32: 0x3525_4B0B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCF9A_30A3,
        process_table_crc32: 0x36B0_7EA9,
        video_crc32: 0xCF3E_E8D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD065_D0D5,
        process_table_crc32: 0xDAAA_76DF,
        video_crc32: 0x463B_31DF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7127_A7E7,
        process_table_crc32: 0x181C_6397,
        video_crc32: 0x18F5_FC8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA8A7_5523,
        process_table_crc32: 0xF6EF_6818,
        video_crc32: 0xA0ED_A907,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3471_8C0B,
        process_table_crc32: 0x1D17_28E8,
        video_crc32: 0x6789_052E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB27B_71F7,
        process_table_crc32: 0xFED1_B8BF,
        video_crc32: 0xD3DB_B46F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB1E5_F3E0,
        process_table_crc32: 0xA534_4040,
        video_crc32: 0x3F62_DA1B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBCEA_255F,
        process_table_crc32: 0x80BC_F248,
        video_crc32: 0xDF81_3257,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x77B3_DF52,
        process_table_crc32: 0x9946_1A02,
        video_crc32: 0x2609_91B6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x684C_3F24,
        process_table_crc32: 0xDD59_90B8,
        video_crc32: 0x23CB_D15C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2195_7543,
        process_table_crc32: 0x9087_304E,
        video_crc32: 0x3EAF_A26C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8401_2A32,
        process_table_crc32: 0x7341_A019,
        video_crc32: 0xC0B4_892B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x18D7_F31A,
        process_table_crc32: 0x3E58_2BE4,
        video_crc32: 0x9A4D_5019,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7646_33B3,
        process_table_crc32: 0x8274_E74D,
        video_crc32: 0xBA4B_9DF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x09CC_1C11,
        process_table_crc32: 0x2B4A_F501,
        video_crc32: 0x90AB_9493,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE61F_8B10,
        process_table_crc32: 0x17CB_CBB9,
        video_crc32: 0x8BF7_4439,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA97B_87C2,
        process_table_crc32: 0xBFCE_4975,
        video_crc32: 0x10D7_2C3D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB684_67B4,
        process_table_crc32: 0x5C08_D922,
        video_crc32: 0x282C_6251,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x090F_95BB,
        process_table_crc32: 0xC1A6_0017,
        video_crc32: 0x5174_557E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0385_8C80,
        process_table_crc32: 0x2F55_0B98,
        video_crc32: 0x0E08_2D0F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9F53_55A8,
        process_table_crc32: 0x52CC_7266,
        video_crc32: 0x54FB_EC46,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1959_A854,
        process_table_crc32: 0x33A1_41FA,
        video_crc32: 0xE558_38C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1AC7_2A43,
        process_table_crc32: 0x6776_6DBC,
        video_crc32: 0x5D92_0A43,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D8F_8017,
        process_table_crc32: 0x84B0_FDEB,
        video_crc32: 0xA817_3944,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2EFF_2170,
        process_table_crc32: 0xCF67_A8C6,
        video_crc32: 0xECB3_A0A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3100_C106,
        process_table_crc32: 0x7E6B_A6BA,
        video_crc32: 0x3ACE_002A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x78D9_8B61,
        process_table_crc32: 0x237D_A0B0,
        video_crc32: 0xEA7B_99B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDD4D_D410,
        process_table_crc32: 0xDF5C_6AC0,
        video_crc32: 0x82F6_41A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x419B_0D38,
        process_table_crc32: 0x31AF_614F,
        video_crc32: 0xE527_32CA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1B1B_F8A2,
        process_table_crc32: 0x6A06_2A86,
        video_crc32: 0x0518_0C4A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCB37_AF4F,
        process_table_crc32: 0x89C0_BAD1,
        video_crc32: 0x5738_BB45,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF756_EDB4,
        process_table_crc32: 0x0D8B_15DF,
        video_crc32: 0x7406_E674,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0259_5E61,
        process_table_crc32: 0x310A_2B67,
        video_crc32: 0x5D29_AC07,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x747F_D4EA,
        process_table_crc32: 0x7DEF_1B11,
        video_crc32: 0xDE7A_D6D9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7A9B_DB97,
        process_table_crc32: 0x39F0_91AB,
        video_crc32: 0xBBA3_4810,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0CBD_511C,
        process_table_crc32: 0x742E_315D,
        video_crc32: 0x8E10_4C7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF9B2_E2C9,
        process_table_crc32: 0x97E8_A10A,
        video_crc32: 0xA03D_AB0A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC5D3_A032,
        process_table_crc32: 0x61D0_5285,
        video_crc32: 0xF73F_73E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x15FF_F7DF,
        process_table_crc32: 0x67E1_2A90,
        video_crc32: 0x12DA_FB50,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD726_B0D5,
        process_table_crc32: 0xA557_3FD8,
        video_crc32: 0x1AD7_3E65,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2229_0300,
        process_table_crc32: 0x80DF_8DD0,
        video_crc32: 0x1264_E62A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x540F_898B,
        process_table_crc32: 0x31D3_83AC,
        video_crc32: 0x2F8C_BFDF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCE64_1616,
        process_table_crc32: 0xD215_13FB,
        video_crc32: 0xA133_5273,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB842_9C9D,
        process_table_crc32: 0xB58B_DE92,
        video_crc32: 0x867E_E1A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4D4D_2F48,
        process_table_crc32: 0x5B78_D51D,
        video_crc32: 0x6F78_7447,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE87C_1830,
        process_table_crc32: 0x25DD_7008,
        video_crc32: 0x8C0D_FEAA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3850_4FDD,
        process_table_crc32: 0x5DB9_CF24,
        video_crc32: 0x821A_CE63,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0431_0D26,
        process_table_crc32: 0x1067_6FD2,
        video_crc32: 0xB2B6_BBD7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF13E_BEF3,
        process_table_crc32: 0xF3A1_FF85,
        video_crc32: 0x039F_4610,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8718_3478,
        process_table_crc32: 0xD6D4_F325,
        video_crc32: 0x074B_C228,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7B92_1C84,
        process_table_crc32: 0x92CB_799F,
        video_crc32: 0x3D16_B35E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0DB4_960F,
        process_table_crc32: 0x3BF5_6BD3,
        video_crc32: 0x5534_905D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF8BB_25DA,
        process_table_crc32: 0x0FD9_0F9D,
        video_crc32: 0xD5F9_6896,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC4DA_6721,
        process_table_crc32: 0x3EFE_20AC,
        video_crc32: 0xBC5C_4951,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x14F6_30CC,
        process_table_crc32: 0xDD38_B0FB,
        video_crc32: 0x7A81_78DE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBC18_E2D7,
        process_table_crc32: 0x7CED_5C58,
        video_crc32: 0x5BAB_B0D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4917_5102,
        process_table_crc32: 0x5965_EE50,
        video_crc32: 0xB9A4_84CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3F31_DB89,
        process_table_crc32: 0x7E08_D922,
        video_crc32: 0x63BD_16BF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA55A_4414,
        process_table_crc32: 0x231E_DF28,
        video_crc32: 0x57E0_1B5E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_CE9F,
        process_table_crc32: 0x77C9_F36E,
        video_crc32: 0x4DC2_B4D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2673_7D4A,
        process_table_crc32: 0x940F_6339,
        video_crc32: 0x0CE3_C26B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x435E_C193,
        process_table_crc32: 0xD775_6CE2,
        video_crc32: 0x41E6_421C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9372_967E,
        process_table_crc32: 0xD8CB_F88A,
        video_crc32: 0x964F_AE5E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAF13_D485,
        process_table_crc32: 0x24EA_32FA,
        video_crc32: 0x428D_7E26,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5A1C_6750,
        process_table_crc32: 0x186B_0C42,
        video_crc32: 0x09B0_4594,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2C3A_EDDB,
        process_table_crc32: 0xB06E_8E8E,
        video_crc32: 0x8436_47E5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x22DE_E2A6,
        process_table_crc32: 0x53A8_1ED9,
        video_crc32: 0x6CED_9CCA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x54F8_682D,
        process_table_crc32: 0xEB98_8441,
        video_crc32: 0xB337_85CF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA1F7_DBF8,
        process_table_crc32: 0xDFB4_E00F,
        video_crc32: 0xD0A2_F3FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9D96_9903,
        process_table_crc32: 0x0A73_7D84,
        video_crc32: 0xB754_3E0C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4DBA_CEEE,
        process_table_crc32: 0x7217_C2A8,
        video_crc32: 0x8801_AD91,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x173A_3B74,
        process_table_crc32: 0x3FC9_625E,
        video_crc32: 0x03BC_68B6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE235_88A1,
        process_table_crc32: 0xDC0F_F209,
        video_crc32: 0x5075_9DA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9413_022A,
        process_table_crc32: 0x6DE8_B378,
        video_crc32: 0x02AA_BAB2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E78_9DB7,
        process_table_crc32: 0xDCE4_BD04,
        video_crc32: 0xE73D_FE18,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x785E_173C,
        process_table_crc32: 0x81F2_BB0E,
        video_crc32: 0xC380_FE33,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8D51_A4E9,
        process_table_crc32: 0x4344_AE46,
        video_crc32: 0x2D1B_62B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x25BF_76F2,
        process_table_crc32: 0x6E61_46B8,
        video_crc32: 0xCA4F_E077,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF593_211F,
        process_table_crc32: 0x7BE4_E6EA,
        video_crc32: 0xDA8D_014A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC9F2_63E4,
        process_table_crc32: 0x9822_76BD,
        video_crc32: 0x4B5A_CBE3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3CFD_D031,
        process_table_crc32: 0xC3C7_8E42,
        video_crc32: 0x463B_0D1C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4ADB_5ABA,
        process_table_crc32: 0xE64F_3C4A,
        video_crc32: 0x7D6A_E6DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF6A6_A983,
        process_table_crc32: 0xFFB5_D400,
        video_crc32: 0x6D9C_262F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8080_2308,
        process_table_crc32: 0xBBAA_5EBA,
        video_crc32: 0x928F_1667,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x758F_90DD,
        process_table_crc32: 0xF674_FE4C,
        video_crc32: 0x3AD5_C25A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x49EE_D226,
        process_table_crc32: 0x15B2_6E1B,
        video_crc32: 0x1489_E148,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x99C2_85CB,
        process_table_crc32: 0x906F_BA81,
        video_crc32: 0xE54F_D640,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x312C_57D0,
        process_table_crc32: 0xD92C_2A9C,
        video_crc32: 0xBA25_96CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC423_E405,
        process_table_crc32: 0x7012_38D0,
        video_crc32: 0x705C_E245,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB205_6E8E,
        process_table_crc32: 0x4C93_0668,
        video_crc32: 0x8509_2997,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x286E_F113,
        process_table_crc32: 0xE496_84A4,
        video_crc32: 0x68CD_AE5B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E48_7B98,
        process_table_crc32: 0x0750_14F3,
        video_crc32: 0x7FAB_0823,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAB47_C84D,
        process_table_crc32: 0x9AFE_CDC6,
        video_crc32: 0x6023_0153,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF1C7_3DD7,
        process_table_crc32: 0xAEAD_CECC,
        video_crc32: 0xA346_F403,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x21EB_6A3A,
        process_table_crc32: 0x4A29_3AFF,
        video_crc32: 0xAA15_4D80,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D8A_28C1,
        process_table_crc32: 0x1982_EAB1,
        video_crc32: 0x7707_8062,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE885_9B14,
        process_table_crc32: 0x90CD_FB03,
        video_crc32: 0x4CE7_3CC4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9EA3_119F,
        process_table_crc32: 0x061C_209C,
        video_crc32: 0x0253_C08C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9047_1EE2,
        process_table_crc32: 0x7853_F5E5,
        video_crc32: 0x9826_0EAD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE661_9469,
        process_table_crc32: 0x0C92_B1F6,
        video_crc32: 0x354A_4346,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x136E_27BC,
        process_table_crc32: 0x2D2B_4672,
        video_crc32: 0xB2E2_D7C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F0F_6547,
        process_table_crc32: 0x66F0_FA98,
        video_crc32: 0x4153_4C61,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF23_32AA,
        process_table_crc32: 0x0D1C_BB15,
        video_crc32: 0xA9BD_7021,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9A0E_8E73,
        process_table_crc32: 0x5EF4_0272,
        video_crc32: 0x973D_962F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6F01_3DA6,
        process_table_crc32: 0x0727_9088,
        video_crc32: 0x126F_548E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1927_B72D,
        process_table_crc32: 0x4EB1_E5F8,
        video_crc32: 0xD83E_FD02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x834C_28B0,
        process_table_crc32: 0xB0E3_1830,
        video_crc32: 0x8AD2_5105,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF56A_A23B,
        process_table_crc32: 0xDD2B_D093,
        video_crc32: 0xF6C0_5E9A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0065_11EE,
        process_table_crc32: 0x4D6D_4D91,
        video_crc32: 0x591A_DDEF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA88B_C3F5,
        process_table_crc32: 0x4F0E_67F1,
        video_crc32: 0x4679_9BDF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x78A7_9418,
        process_table_crc32: 0xF644_7C76,
        video_crc32: 0x26A6_1F55,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x44C6_D6E3,
        process_table_crc32: 0xD70A_6105,
        video_crc32: 0x3AD8_989D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB1C9_6536,
        process_table_crc32: 0xC824_49B9,
        video_crc32: 0x45A0_32FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC7EF_EFBD,
        process_table_crc32: 0x98BB_B079,
        video_crc32: 0x310E_5588,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3B65_C741,
        process_table_crc32: 0xC165_B701,
        video_crc32: 0x6337_2EA2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4D43_4DCA,
        process_table_crc32: 0x0B74_654F,
        video_crc32: 0xF7C7_F3D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEFE1_E150,
        process_table_crc32: 0x10D2_39AF,
        video_crc32: 0xE45F_2136,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xAE6B_9A0F,
        video_crc32: 0x71CB_8D14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xC001_3D66,
        video_crc32: 0xCC3F_8830,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xF262_8368,
        video_crc32: 0x40AF_125C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xF5BB_22BC,
        video_crc32: 0x8C90_34D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xC874_B18D,
        video_crc32: 0xA466_B760,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x4504_3FE1,
        video_crc32: 0x420B_E281,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xF837_BF49,
        video_crc32: 0x6F73_7A73,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x3D10_35D3,
        video_crc32: 0x8F88_1781,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xDE29_26BD,
        video_crc32: 0x5E3E_530E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x1DBE_2C9D,
        video_crc32: 0x8534_0E2D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xDB22_6DC2,
        video_crc32: 0x2E5F_C042,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x72E3_FCB7,
        video_crc32: 0x73FF_97AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x87E1_3C52,
        video_crc32: 0x519C_DBAA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x62B0_A058,
        video_crc32: 0x2DA1_66C5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x9BD4_944C,
        video_crc32: 0x1C22_E6DB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x8CC5_9364,
        video_crc32: 0xE7EC_2B47,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x9215_BE00,
        video_crc32: 0x46A2_D647,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x611C_2C2B,
        video_crc32: 0x7022_FEFE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x9724_DFA4,
        video_crc32: 0x7599_2ED0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x272C_81DE,
        video_crc32: 0xE335_9994,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x0C46_B360,
        video_crc32: 0xB421_D67C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xBA60_7832,
        video_crc32: 0xB10D_3DEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x58E8_45EC,
        video_crc32: 0x7333_E625,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xE5DB_C544,
        video_crc32: 0x0358_1808,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x8725_5533,
        video_crc32: 0xC160_F44F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x72CB_6F5D,
        video_crc32: 0xEA5F_6B50,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x7512_CE89,
        video_crc32: 0xD71A_079A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x6C5A_AB68,
        video_crc32: 0x7AFC_9E20,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xA69A_8942,
        video_crc32: 0x4448_319F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xA143_2896,
        video_crc32: 0x3EDA_9040,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x9C8C_BBA7,
        video_crc32: 0x7890_82F5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x2D6B_FAD6,
        video_crc32: 0x2B68_6DF1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x557F_F0E4,
        video_crc32: 0x4A0A_C3D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x3FC1_5C6A,
        video_crc32: 0xB317_DDD1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xBAAB_523F,
        video_crc32: 0xA8B8_FB5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x7C37_1360,
        video_crc32: 0xC663_67FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xD5F6_8215,
        video_crc32: 0x9464_6CFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xDAC4_56AC,
        video_crc32: 0x39EE_C9CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x3F95_CAA6,
        video_crc32: 0x1405_3608,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x82A6_4A0E,
        video_crc32: 0x4AED_00DE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xF866_218A,
        video_crc32: 0x5928_DE4D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xF1A7_0BC6,
        video_crc32: 0xE12F_3F1B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x3F05_9A3E,
        video_crc32: 0xB7B7_9637,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xBAD8_4EA4,
        video_crc32: 0xDE09_A03D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xA69B_603A,
        video_crc32: 0xF24A_6734,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x77C1_CD5A,
        video_crc32: 0xFA84_6D44,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x9549_F084,
        video_crc32: 0x0507_4759,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0xE345_712E,
        video_crc32: 0x2B96_73DA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1D78_0294,
        process_table_crc32: 0x4A84_E05B,
        video_crc32: 0x645A_1B28,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x83FD_1528,
        video_crc32: 0xDFFF_9A83,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x6311_31BF,
        video_crc32: 0xB68B_4512,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x33EC_49EE,
        video_crc32: 0x963C_BE14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x1886_7B50,
        video_crc32: 0xB9D4_A1BF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x0656_5634,
        video_crc32: 0x42F0_F672,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x3B99_C505,
        video_crc32: 0x4950_3A91,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x704E_9028,
        video_crc32: 0x6D2B_E66A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x1F4B_9D32,
        video_crc32: 0x6D2B_E66A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x085A_9A1A,
        video_crc32: 0x7C4A_9140,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0xBE7C_5148,
        video_crc32: 0x0DCA_4D25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x7DEB_5B68,
        video_crc32: 0x06F8_DCBA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0xBB77_1A37,
        video_crc32: 0x6125_FEC9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x12B6_8B42,
        video_crc32: 0x994E_C8B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0xDCFA_256E,
        video_crc32: 0x03FE_2932,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0x3E72_18B0,
        video_crc32: 0x0631_5824,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0xAC9E_2BA0,
        video_crc32: 0xC615_DBD5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10A7_E7F7,
        process_table_crc32: 0xA286_A038,
        video_crc32: 0xBC5D_78C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2257_8C2A,
        process_table_crc32: 0x27DD_3AD1,
        video_crc32: 0x1C21_F9C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x134F_E777,
        process_table_crc32: 0xC41B_AA86,
        video_crc32: 0x12DF_5FA0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x79CC_6A7C,
        process_table_crc32: 0x3223_5909,
        video_crc32: 0xBC30_1331,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF64A_D8E3,
        process_table_crc32: 0x321C_120D,
        video_crc32: 0x159A_64B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x649F_C14F,
        process_table_crc32: 0x5371_2191,
        video_crc32: 0x8575_E452,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE829_9E47,
        process_table_crc32: 0x05C9_369D,
        video_crc32: 0xAF67_6F1C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3478_1B01,
        process_table_crc32: 0x2041_8495,
        video_crc32: 0x2151_F1D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3901_58C5,
        process_table_crc32: 0x9D72_043D,
        video_crc32: 0xDF1B_B878,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4CC6_90B7,
        process_table_crc32: 0xE685_18FA,
        video_crc32: 0x106C_049D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB0C9_4AC7,
        process_table_crc32: 0xD5CC_F9D5,
        video_crc32: 0xF3A4_B6A5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1A40_91D7,
        process_table_crc32: 0x088B_DE91,
        video_crc32: 0x0FF7_E5FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6D22_3A19,
        process_table_crc32: 0x2CE6_78DA,
        video_crc32: 0xEE60_5508,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC00A_CDB2,
        process_table_crc32: 0x5482_C7F6,
        video_crc32: 0xC67B_B26E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4CBC_92BA,
        process_table_crc32: 0x8D52_6544,
        video_crc32: 0x74E7_5E0A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x719E_1D0C,
        process_table_crc32: 0x6E94_F513,
        video_crc32: 0x7BBA_BE35,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x69B3_FB35,
        process_table_crc32: 0xE3E4_7B7F,
        video_crc32: 0x0556_5BBE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1C74_3347,
        process_table_crc32: 0x9BF0_714D,
        video_crc32: 0x41CF_F2FA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0108_E3C7,
        process_table_crc32: 0x6806_605F,
        video_crc32: 0x51CE_2E72,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBED5_9D2A,
        process_table_crc32: 0x86F5_6BD0,
        video_crc32: 0x1B14_9340,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3153_2FB5,
        process_table_crc32: 0x0A6E_2BAD,
        video_crc32: 0x80BA_8B6F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA386_3619,
        process_table_crc32: 0xE9A8_BBFA,
        video_crc32: 0x403D_41B0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F30_6911,
        process_table_crc32: 0x1CAA_7B1F,
        video_crc32: 0x85C1_9E8E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD9D8_D8B4,
        process_table_crc32: 0xF9FB_E715,
        video_crc32: 0xE54F_BDC4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAB5A_477E,
        process_table_crc32: 0x591C_5D87,
        video_crc32: 0x8907_6E36,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDE9D_8F0C,
        process_table_crc32: 0x178E_D429,
        video_crc32: 0xFD18_8373,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2292_557C,
        process_table_crc32: 0x243A_F8E2,
        video_crc32: 0xFAC6_A274,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x881B_8E6C,
        process_table_crc32: 0xC7FC_68B5,
        video_crc32: 0xA4F7_0AF1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE6EE_3603,
        process_table_crc32: 0x31C4_9B3A,
        video_crc32: 0x2E50_1F03,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x616F_8A52,
        process_table_crc32: 0xCBCB_C462,
        video_crc32: 0x2BA6_460D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEDD9_D55A,
        process_table_crc32: 0xAAA6_F7FE,
        video_crc32: 0xFDAC_F8C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD0FB_5AEC,
        process_table_crc32: 0xC289_3FCA,
        video_crc32: 0x24DC_5E2E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC8D6_BCD5,
        process_table_crc32: 0xFE08_0172,
        video_crc32: 0xCBCB_87A5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBD11_74A7,
        process_table_crc32: 0xC203_81FA,
        video_crc32: 0x4A17_1A80,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x129C_E0B0,
        process_table_crc32: 0x21C5_11AD,
        video_crc32: 0xB1BD_E1B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5F07_EA7E,
        process_table_crc32: 0xD42B_2BC3,
        video_crc32: 0x7A1A_5FD4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3793_893F,
        process_table_crc32: 0xA034_8B0E,
        video_crc32: 0x2907_C89F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x31DD_29A2,
        process_table_crc32: 0xC84F_79C9,
        video_crc32: 0x32D6_1402,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCEE2_1E45,
        process_table_crc32: 0xB02B_C6E5,
        video_crc32: 0xA92F_1ABE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE0F5_EF20,
        process_table_crc32: 0x69FB_6457,
        video_crc32: 0x4F09_4C7E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1FCA_D8C7,
        process_table_crc32: 0x8A3D_F400,
        video_crc32: 0x94D8_91DC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x1984_785A,
        process_table_crc32: 0x3BDA_B571,
        video_crc32: 0x7855_37B4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7110_1B1B,
        process_table_crc32: 0x1ED8_B949,
        video_crc32: 0x4B2E_63DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3C8B_11D5,
        process_table_crc32: 0x43CE_BF43,
        video_crc32: 0xB19E_F5C3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFE07_19B3,
        process_table_crc32: 0xE61B_AA86,
        video_crc32: 0xE2E8_B028,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF849_B92E,
        process_table_crc32: 0x08E8_A109,
        video_crc32: 0xF03E_1C4C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0776_8EC9,
        process_table_crc32: 0x8473_E174,
        video_crc32: 0x53CB_53B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDD46_D0A1,
        process_table_crc32: 0x67B5_7123,
        video_crc32: 0xA952_69CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2279_E746,
        process_table_crc32: 0x6887_A59A,
        video_crc32: 0xDCA6_AEF6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2437_47DB,
        process_table_crc32: 0x8DD6_3990,
        video_crc32: 0x2C74_A594,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB884_8B97,
        process_table_crc32: 0x0022_D39E,
        video_crc32: 0x79C2_EB61,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF51F_8159,
        process_table_crc32: 0x443D_5924,
        video_crc32: 0x2F6B_D200,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7B90_1E2E,
        process_table_crc32: 0x532B_FA8C,
        video_crc32: 0xE9E8_8197,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7DDE_BEB3,
        process_table_crc32: 0xB0ED_6ADB,
        video_crc32: 0xBE0A_45F2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x82E1_8954,
        process_table_crc32: 0x46D5_9954,
        video_crc32: 0x7776_B171,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9FC8_206A,
        process_table_crc32: 0x1B10_2ED1,
        video_crc32: 0x6B42_9C09,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x60F7_178D,
        process_table_crc32: 0x2620_3ED9,
        video_crc32: 0x03CD_2C9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x66B9_B710,
        process_table_crc32: 0x1AA1_0061,
        video_crc32: 0xBB60_1BA3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0E2D_D451,
        process_table_crc32: 0x26AA_80E9,
        video_crc32: 0xEF90_260C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x43B6_DE9F,
        process_table_crc32: 0xC56C_10BE,
        video_crc32: 0xE875_E0D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF05_12D3,
        process_table_crc32: 0x3FA1_C906,
        video_crc32: 0xA3C0_A4DA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD94B_B24E,
        process_table_crc32: 0x1284_21F8,
        video_crc32: 0xC4B8_7E37,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2674_85A9,
        process_table_crc32: 0xCBA8_BBFA,
        video_crc32: 0x27EA_49D3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFC44_DBC1,
        process_table_crc32: 0xAAC5_8866,
        video_crc32: 0x63D6_7CDD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x037B_EC26,
        process_table_crc32: 0x6A1C_A664,
        video_crc32: 0x3EEB_1002,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0535_4CBB,
        process_table_crc32: 0x89DA_3633,
        video_crc32: 0x2AF7_0E2A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCCC4_681A,
        process_table_crc32: 0xE70F_6F26,
        video_crc32: 0xDD5C_2A46,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x815F_62D4,
        process_table_crc32: 0xBA19_692C,
        video_crc32: 0x39C5_64D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE9CB_0195,
        process_table_crc32: 0x67A6_A7A4,
        video_crc32: 0x4C0F_FF5F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEF85_A108,
        process_table_crc32: 0x538A_C3EA,
        video_crc32: 0x0499_6EB8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10BA_96EF,
        process_table_crc32: 0x05CE_EC56,
        video_crc32: 0x340C_EDB3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3EAD_678A,
        process_table_crc32: 0xE608_7C01,
        video_crc32: 0x1D02_977E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC192_506D,
        process_table_crc32: 0xF64D_D14B,
        video_crc32: 0x871B_4050,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC7DC_F0F0,
        process_table_crc32: 0xCACC_EFF3,
        video_crc32: 0x04E6_71CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAF48_93B1,
        process_table_crc32: 0x1227_DDC1,
        video_crc32: 0x7026_B68F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE2D3_997F,
        process_table_crc32: 0x5638_577B,
        video_crc32: 0xB040_6498,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4D5E_0D68,
        process_table_crc32: 0x7C85_F700,
        video_crc32: 0x748A_CCED,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4B10_ADF5,
        process_table_crc32: 0x9F43_6757,
        video_crc32: 0x748A_CCED,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB42F_9A12,
        process_table_crc32: 0x697B_94D8,
        video_crc32: 0xCF5B_E398,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6E1F_C47A,
        process_table_crc32: 0x0829_EC40,
        video_crc32: 0xDEFE_E846,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9120_F39D,
        process_table_crc32: 0x5E91_FB4C,
        video_crc32: 0xF236_E37C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x976E_5300,
        process_table_crc32: 0x7B19_4944,
        video_crc32: 0xE4CB_797C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0BDD_9F4C,
        process_table_crc32: 0x5E1B_457C,
        video_crc32: 0x653E_10FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4646_9582,
        process_table_crc32: 0xBDDD_D52B,
        video_crc32: 0x1268_17AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2ED2_F6C3,
        process_table_crc32: 0x808B_1B1C,
        video_crc32: 0xE555_97E7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x289C_565E,
        process_table_crc32: 0xADAE_F3E2,
        video_crc32: 0xEF2E_D29A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD7A3_61B9,
        process_table_crc32: 0x4A15_B6D8,
        video_crc32: 0xC332_5713,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x600F_195D,
        process_table_crc32: 0x3271_09F4,
        video_crc32: 0x1A6D_1E84,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9F30_2EBA,
        process_table_crc32: 0xEBA1_AB46,
        video_crc32: 0xF21B_6871,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x997E_8E27,
        process_table_crc32: 0x0867_3B11,
        video_crc32: 0xE981_A2AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF1EA_ED66,
        process_table_crc32: 0x8517_B57D,
        video_crc32: 0xA078_E8B1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBC71_E7A8,
        process_table_crc32: 0xB91C_35F5,
        video_crc32: 0xBE65_F6F0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x20C2_2BE4,
        process_table_crc32: 0xFD03_BF4F,
        video_crc32: 0x4C71_1797,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x268C_8B79,
        process_table_crc32: 0x335E_AD8E,
        video_crc32: 0x9C4E_419E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD9B3_BC9E,
        process_table_crc32: 0x0772_C9C0,
        video_crc32: 0x72B2_6D6C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0383_E2F6,
        process_table_crc32: 0x5136_E67C,
        video_crc32: 0x5AA4_F193,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFCBC_D511,
        process_table_crc32: 0xB2F0_762B,
        video_crc32: 0xE1D6_FF9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFAF2_758C,
        process_table_crc32: 0x872B_98CC,
        video_crc32: 0xE1D6_FF9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x557F_E19B,
        process_table_crc32: 0xA2A3_2AC4,
        video_crc32: 0xAB58_A418,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x18E4_EB55,
        process_table_crc32: 0x11C0_1FF2,
        video_crc32: 0x843F_621B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7070_8814,
        process_table_crc32: 0x55A0_F20C,
        video_crc32: 0xC510_F714,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x763E_2889,
        process_table_crc32: 0x3CDF_B07B,
        video_crc32: 0x28E9_6577,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8901_1F6E,
        process_table_crc32: 0xEDDF_C3FE,
        video_crc32: 0x5CAF_F275,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA716_EE0B,
        process_table_crc32: 0xAEA5_CC25,
        video_crc32: 0x610E_39EC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5829_D9EC,
        process_table_crc32: 0x280A_2A89,
        video_crc32: 0x5862_2319,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5E67_7971,
        process_table_crc32: 0xC4B1_6C95,
        video_crc32: 0x3EE0_4021,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x36F3_1A30,
        process_table_crc32: 0xD1E7_1034,
        video_crc32: 0x8FC1_E949,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7B68_10FE,
        process_table_crc32: 0x3074_AD48,
        video_crc32: 0xF412_7154,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB299_345F,
        process_table_crc32: 0xAC45_E5BB,
        video_crc32: 0xC743_C9F7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB4D7_94C2,
        process_table_crc32: 0x29DD_1112,
        video_crc32: 0xFFE4_0252,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4BE8_A325,
        process_table_crc32: 0x7700_33AD,
        video_crc32: 0x2CFF_C6C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x91D8_FD4D,
        process_table_crc32: 0xC2E3_FC9E,
        video_crc32: 0xB444_C0EF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6EE7_CAAA,
        process_table_crc32: 0xCF90_087A,
        video_crc32: 0x344C_A698,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x68A9_6A37,
        process_table_crc32: 0xA4F7_6776,
        video_crc32: 0xC79E_25C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF41A_A67B,
        process_table_crc32: 0x6EE6_B538,
        video_crc32: 0x65AC_5FEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB981_ACB5,
        process_table_crc32: 0x279B_C585,
        video_crc32: 0x1D30_7C8B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD115_CFF4,
        process_table_crc32: 0x9B28_79B8,
        video_crc32: 0xB023_4D60,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD75B_6F69,
        process_table_crc32: 0x6436_02C1,
        video_crc32: 0x2E61_7755,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2864_588E,
        process_table_crc32: 0xB803_EA9C,
        video_crc32: 0x15A7_4C12,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x354D_F1B0,
        process_table_crc32: 0xE900_9715,
        video_crc32: 0xE122_B9A2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCA72_C657,
        process_table_crc32: 0x7FD1_4C8A,
        video_crc32: 0x3ADE_855A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCC3C_66CA,
        process_table_crc32: 0x58E8_BF35,
        video_crc32: 0xA92D_4BB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA4A8_058B,
        process_table_crc32: 0x54B7_4F24,
        video_crc32: 0xDE8C_1897,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2624_4B19,
        process_table_crc32: 0x4E66_4B14,
        video_crc32: 0xF9F0_EDBA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xB12E_1636,
        video_crc32: 0xB96B_F103,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xB6F7_B7E2,
        video_crc32: 0xCCFF_7086,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x1F36_2697,
        video_crc32: 0xCC20_48A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xE90E_D518,
        video_crc32: 0x8B35_BD72,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x9AEB_F20D,
        video_crc32: 0x3723_4C7C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x20C5_60D9,
        video_crc32: 0x0EF8_9CFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xC3FC_73B7,
        video_crc32: 0x8C36_4281,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xB57A_4C2D,
        video_crc32: 0x7BBD_2374,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xC376_CD87,
        video_crc32: 0xBD2D_4FB5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x0DD4_5C7F,
        video_crc32: 0x22A2_9112,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xC4AD_A90C,
        video_crc32: 0x9DFA_3257,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xDA7D_8468,
        video_crc32: 0x48F1_6A03,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xBD5F_6DB4,
        video_crc32: 0x0F0C_7BF0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x13BB_65A5,
        video_crc32: 0x0B79_057C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x8FC0_EB0A,
        video_crc32: 0x0BBB_7CBA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x2601_7A7F,
        video_crc32: 0xFC2C_6476,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x6DD6_2F52,
        video_crc32: 0x59B0_43A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xD0E5_AFFA,
        video_crc32: 0x25B1_C9EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x02D3_2248,
        video_crc32: 0xCE9C_EF8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xF92C_ED6C,
        video_crc32: 0x84AC_F2A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x5DD8_E7C1,
        video_crc32: 0x1392_07B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x8707_8800,
        video_crc32: 0x6905_C191,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xA68B_35AF,
        video_crc32: 0x14DC_8746,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x6F1E_3A57,
        video_crc32: 0xB331_73B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x68C7_9B83,
        video_crc32: 0x189B_99DC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x4992_9F01,
        video_crc32: 0xC281_C686,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xE5D6_1C1C,
        video_crc32: 0x5D5A_9E1F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xE20F_BDC8,
        video_crc32: 0xCFDF_BA76,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x4BCE_2CBD,
        video_crc32: 0x59C4_5270,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xBDF6_DF32,
        video_crc32: 0xAEDD_FF73,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xF284_373A,
        video_crc32: 0x88E5_2EA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x48AA_A5EE,
        video_crc32: 0x4E68_6CBC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xC01B_B184,
        video_crc32: 0x8968_8DB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xAF94_02AE,
        video_crc32: 0xA417_DDEF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x12A7_8206,
        video_crc32: 0xEB6A_A4C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x229E_B750,
        video_crc32: 0xFF89_5057,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xAAC1_22DD,
        video_crc32: 0x5A3F_6D51,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x8758_EE96,
        video_crc32: 0xEABD_37EC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x23AC_E43B,
        video_crc32: 0x9E25_3F9E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xE07A_074A,
        video_crc32: 0xC68D_F8B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xEBAB_FF18,
        video_crc32: 0x9F37_96FB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xEC72_5ECC,
        video_crc32: 0x5BF8_CB55,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x45B3_CFB9,
        video_crc32: 0x8857_DD31,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xC8C3_41D5,
        video_crc32: 0x7A10_0778,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xBECF_C07F,
        video_crc32: 0x646D_DE73,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x348D_5804,
        video_crc32: 0x77FF_E3C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x9079_52A9,
        video_crc32: 0xE0E0_0260,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x4AA6_3D68,
        video_crc32: 0x1D16_A50E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x6B2A_80C7,
        video_crc32: 0x0255_4844,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x9E28_4022,
        video_crc32: 0x9011_4AB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0xEF77_DE6C,
        video_crc32: 0x9CCA_D82B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD37C_BF09,
        process_table_crc32: 0x5244_5EC4,
        video_crc32: 0xD498_FC81,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x5BCA_EE0E,
        video_crc32: 0xE458_EAAD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x451A_C36A,
        video_crc32: 0x28CB_1717,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xECDB_521F,
        video_crc32: 0x7CDF_B269,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x1AE3_A190,
        video_crc32: 0x68B9_1066,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xAAEB_FFEA,
        video_crc32: 0x44E3_CAF9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x158F_CF10,
        video_crc32: 0x70F2_7425,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xA3A9_0442,
        video_crc32: 0x4602_DC71,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xD52F_3BD8,
        video_crc32: 0x17FA_A65F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xA323_BA72,
        video_crc32: 0x99F8_7291,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x6D81_2B8A,
        video_crc32: 0x54E1_4718,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x986F_11E4,
        video_crc32: 0xD78B_C3CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x3B42_BA9D,
        video_crc32: 0x264E_F064,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x193E_4978,
        video_crc32: 0x5F70_3221,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xBF53_F532,
        video_crc32: 0x4EEB_D4FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xB88A_54E6,
        video_crc32: 0x8EE1_800B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x114B_C593,
        video_crc32: 0x7CDB_8205,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0xA0AC_84E2,
        video_crc32: 0x9810_C4FD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x3567_433F,
        process_table_crc32: 0x1D9F_044A,
        video_crc32: 0x281C_9E5B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB300_0862,
        process_table_crc32: 0x103D_B4CF,
        video_crc32: 0x36C0_8B10,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD9C0_E171,
        process_table_crc32: 0xD28B_A187,
        video_crc32: 0x850F_F8C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x15FC_A5DE,
        process_table_crc32: 0x3C78_AA08,
        video_crc32: 0x1BB0_774C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x17DF_6035,
        process_table_crc32: 0xFFAE_4979,
        video_crc32: 0xD72E_E229,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBEFD_4DD9,
        process_table_crc32: 0xC72B_7866,
        video_crc32: 0xBEF5_A8B1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBB3F_5D42,
        process_table_crc32: 0xC819_ACDF,
        video_crc32: 0x16C6_962F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFAC8_81D8,
        process_table_crc32: 0x9CCE_8099,
        video_crc32: 0x83FD_5017,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFB75_65BC,
        process_table_crc32: 0x53D1_D812,
        video_crc32: 0xCD8A_35EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDB3E_B2F4,
        process_table_crc32: 0x17CE_52A8,
        video_crc32: 0x186F_9D8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9DA9_8A4D,
        process_table_crc32: 0x5A10_F25E,
        video_crc32: 0x7EB4_0071,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD003_DDB8,
        process_table_crc32: 0xB9D6_6209,
        video_crc32: 0xCA6E_1012,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDB64_DD71,
        process_table_crc32: 0x4FEE_9186,
        video_crc32: 0x8832_CA17,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7246_F09D,
        process_table_crc32: 0xFE41_9B6B,
        video_crc32: 0xFE0B_DAE2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBE26_130F,
        process_table_crc32: 0x8625_2447,
        video_crc32: 0xB4E0_351B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF8ED_8C8B,
        process_table_crc32: 0x2F1B_360B,
        video_crc32: 0x9D5C_100C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xF950_68EF,
        process_table_crc32: 0x139A_08B3,
        video_crc32: 0x7AA1_2C6F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x10B9_4CAE,
        process_table_crc32: 0x1C05_9195,
        video_crc32: 0xA489_28A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x5112_3709,
        process_table_crc32: 0x969F_1B32,
        video_crc32: 0x4413_9A33,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9D2E_73A6,
        process_table_crc32: 0x0B31_C207,
        video_crc32: 0x5449_9CB2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9F0D_B64D,
        process_table_crc32: 0xE5C2_C988,
        video_crc32: 0xF232_3D8A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x362F_9BA1,
        process_table_crc32: 0x6B36_B2BF,
        video_crc32: 0xBB72_7F0D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEBB0_ABCF,
        process_table_crc32: 0x0A5B_8123,
        video_crc32: 0xB548_21C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB8CE_FD11,
        process_table_crc32: 0x5E8C_AD65,
        video_crc32: 0xF8B3_CFD6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xB973_1975,
        process_table_crc32: 0xBD4A_3D32,
        video_crc32: 0xBF3B_D3CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9938_CE3D,
        process_table_crc32: 0xF69D_681F,
        video_crc32: 0x1C6D_33EB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDFAF_F684,
        process_table_crc32: 0xE9EA_62A0,
        video_crc32: 0xC26E_A97A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDA31_4122,
        process_table_crc32: 0x15CB_A8D0,
        video_crc32: 0xB746_3ADA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDF2E_C7D7,
        process_table_crc32: 0xFB38_A35F,
        video_crc32: 0xEF93_445A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x760C_EA3B,
        process_table_crc32: 0xE3AD_E166,
        video_crc32: 0x2BAD_E2DF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBA6C_09A9,
        process_table_crc32: 0x006B_7131,
        video_crc32: 0xDA18_CB94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFCA7_962D,
        process_table_crc32: 0xC9FE_7EC9,
        video_crc32: 0xA65F_2554,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFD1A_7249,
        process_table_crc32: 0xB8A1_E087,
        video_crc32: 0x84D4_8F98,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBA6A_C111,
        process_table_crc32: 0xCF74_51A1,
        video_crc32: 0x0619_A8C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFF63_D827,
        process_table_crc32: 0xF367_53BB,
        video_crc32: 0xA417_1A90,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x30C1_BD07,
        process_table_crc32: 0xBEB9_F34D,
        video_crc32: 0x031F_21D6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDD0B_CA84,
        process_table_crc32: 0x5D7F_631A,
        video_crc32: 0x9C33_8500,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x985E_748F,
        process_table_crc32: 0xAB47_9095,
        video_crc32: 0x1530_586A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x993E_0685,
        process_table_crc32: 0x3F76_D9D5,
        video_crc32: 0x49C1_DC8C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xDC6B_B88E,
        process_table_crc32: 0x5E1B_EA49,
        video_crc32: 0x8874_0019,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x31A1_CF0D,
        process_table_crc32: 0x9CAD_FF01,
        video_crc32: 0x19E6_7A07,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFE03_AA2D,
        process_table_crc32: 0xB925_4D09,
        video_crc32: 0xB54A_D436,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBB0A_B31B,
        process_table_crc32: 0xFB44_41BC,
        video_crc32: 0x950B_23CA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x52B2_19AD,
        process_table_crc32: 0x1882_D1EB,
        video_crc32: 0x4244_80FC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xBF78_6E2E,
        process_table_crc32: 0x2BCB_30C4,
        video_crc32: 0xE39B_7384,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xFA2D_D025,
        process_table_crc32: 0x91EF_170D,
        video_crc32: 0xE097_CFB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x35D3_1238,
        process_table_crc32: 0x218C_B302,
        video_crc32: 0xC0F2_BD96,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x7086_AC33,
        process_table_crc32: 0x59E8_0C2E,
        video_crc32: 0x09F3_EEA7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9D4C_DBB0,
        process_table_crc32: 0x1436_ACD8,
        video_crc32: 0x149A_21D0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x9C70_0E87,
        process_table_crc32: 0xF7F0_3C8F,
        video_crc32: 0xC9D7_BB3C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xD979_17B1,
        process_table_crc32: 0x7A80_B2E3,
        video_crc32: 0x45D5_90F3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2A30_A3EC,
        process_table_crc32: 0x1C43_3135,
        video_crc32: 0x45D5_90F3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC7FA_D46F,
        process_table_crc32: 0x585C_BB8F,
        video_crc32: 0x1E23_0F21,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x82AF_6A64,
        process_table_crc32: 0xF162_A9C3,
        video_crc32: 0x6F6F_1DCD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC583_7E01,
        process_table_crc32: 0x1F91_A24C,
        video_crc32: 0xA5E8_6EF9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x80D6_C00A,
        process_table_crc32: 0x0704_E075,
        video_crc32: 0x0F16_63EC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6D1C_B789,
        process_table_crc32: 0xE4C2_7022,
        video_crc32: 0x7B33_3E74,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA2BE_D2A9,
        process_table_crc32: 0x4517_9C81,
        video_crc32: 0x2B76_938C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE7B7_CB9F,
        process_table_crc32: 0x609F_2E89,
        video_crc32: 0xA756_01D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE68B_1EA8,
        process_table_crc32: 0xB49F_1B32,
        video_crc32: 0x9B7A_3310,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0B41_692B,
        process_table_crc32: 0xE989_1D38,
        video_crc32: 0x11D9_0BA9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4E14_D720,
        process_table_crc32: 0xBD5E_317E,
        video_crc32: 0x067C_E6D3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x81EA_153D,
        process_table_crc32: 0x5E98_A129,
        video_crc32: 0x64BB_F3F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC4BF_AB36,
        process_table_crc32: 0xA8A0_52A6,
        video_crc32: 0x6AA3_1B8E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2975_DCB5,
        process_table_crc32: 0xC6A1_0FBA,
        video_crc32: 0xB326_3CA8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE29D_A333,
        process_table_crc32: 0xA7CC_3C26,
        video_crc32: 0xA067_8AEF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA794_BA05,
        process_table_crc32: 0x5BED_F656,
        video_crc32: 0x9F48_2652,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6836_DF25,
        process_table_crc32: 0x676C_C8EE,
        video_crc32: 0xE2D3_378C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x85FC_A8A6,
        process_table_crc32: 0x7AF9_4C9E,
        video_crc32: 0xD3DA_07BC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC0A9_16AD,
        process_table_crc32: 0x993F_DCC9,
        video_crc32: 0x6367_8FD6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC1C9_64A7,
        process_table_crc32: 0x210F_4651,
        video_crc32: 0x2CD3_0A82,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x849C_DAAC,
        process_table_crc32: 0xCFFC_4DDE,
        video_crc32: 0x1BC4_2FB4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6956_AD2F,
        process_table_crc32: 0x3389_BD5D,
        video_crc32: 0x1EC1_182A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA6F4_C80F,
        process_table_crc32: 0x4BED_0271,
        video_crc32: 0x70D0_C73B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE3FD_D139,
        process_table_crc32: 0x0633_A287,
        video_crc32: 0xFFE7_A740,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA48D_6261,
        process_table_crc32: 0xE5F5_32D0,
        video_crc32: 0x281D_13DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x4947_15E2,
        process_table_crc32: 0x1673_7F14,
        video_crc32: 0x39DC_FB03,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0C12_ABE9,
        process_table_crc32: 0x4B65_791E,
        video_crc32: 0xFB21_4987,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC3EC_69F4,
        process_table_crc32: 0x89D3_6C56,
        video_crc32: 0x2937_0E02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x86B9_D7FF,
        process_table_crc32: 0x6720_67D9,
        video_crc32: 0x885D_0EB2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6B73_A07C,
        process_table_crc32: 0x7FB5_25E0,
        video_crc32: 0x39B0_55CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6A4F_754B,
        process_table_crc32: 0x9C73_B5B7,
        video_crc32: 0x0FCC_D9BE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2F46_6C7D,
        process_table_crc32: 0xC796_4D48,
        video_crc32: 0xE1B8_DF46,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE0E4_095D,
        process_table_crc32: 0xE21E_FF40,
        video_crc32: 0xE3B0_6383,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0D2E_7EDE,
        process_table_crc32: 0x3522_1610,
        video_crc32: 0xFA0A_75C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x487B_C0D5,
        process_table_crc32: 0x713D_9CAA,
        video_crc32: 0x0354_0FF5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x418F_8793,
        process_table_crc32: 0x3CE3_3C5C,
        video_crc32: 0xC65D_4426,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x04DA_3998,
        process_table_crc32: 0xDF25_AC0B,
        video_crc32: 0xBCA9_47C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xE910_4E1B,
        process_table_crc32: 0x291D_5F84,
        video_crc32: 0xE8E2_E77C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x26B2_2B3B,
        process_table_crc32: 0xE0D6_EA45,
        video_crc32: 0x217D_18A7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x63BB_320D,
        process_table_crc32: 0x49E8_F809,
        video_crc32: 0xA0FB_0B7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x6287_E73A,
        process_table_crc32: 0x7569_C6B1,
        video_crc32: 0xE82B_270B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8F4D_90B9,
        process_table_crc32: 0x2E01_46B4,
        video_crc32: 0xAFD6_36F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCA18_2EB2,
        process_table_crc32: 0xCDC7_D6E3,
        video_crc32: 0x56AD_5ADA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x05E6_ECAF,
        process_table_crc32: 0x5069_0FD6,
        video_crc32: 0xAEBD_DED3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x40B3_52A4,
        process_table_crc32: 0xBE9A_0459,
        video_crc32: 0xF9AC_FA55,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAD79_2527,
        process_table_crc32: 0x306E_7F6E,
        video_crc32: 0x636C_AFB6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEA09_967F,
        process_table_crc32: 0x5103_4CF2,
        video_crc32: 0xD60E_B834,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAF00_8F49,
        process_table_crc32: 0x05D4_60B4,
        video_crc32: 0x35AF_9E6A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x60A2_EA69,
        process_table_crc32: 0xFF64_1B17,
        video_crc32: 0x428A_B6C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8D68_9DEA,
        process_table_crc32: 0xB4B3_4E3A,
        video_crc32: 0x1C13_5399,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC83D_23E1,
        process_table_crc32: 0x9771_B8D5,
        video_crc32: 0x34DF_BD53,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xC95D_51EB,
        process_table_crc32: 0xC3C9_C9EB,
        video_crc32: 0x02BA_C122,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8C08_EFE0,
        process_table_crc32: 0xE270_3E6F,
        video_crc32: 0x4644_099E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x61C2_9863,
        process_table_crc32: 0x7994_7E28,
        video_crc32: 0x91D2_1DA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAE60_FD43,
        process_table_crc32: 0xE595_B239,
        video_crc32: 0x064C_9174,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEB69_E475,
        process_table_crc32: 0x2F84_6077,
        video_crc32: 0x30B8_AD2B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x2081_9BF3,
        process_table_crc32: 0x7657_F28D,
        video_crc32: 0x9C57_B104,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCD4B_EC70,
        process_table_crc32: 0x3521_1491,
        video_crc32: 0xF78E_BDEA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x881E_527B,
        process_table_crc32: 0xD062_4892,
        video_crc32: 0x84DB_EC80,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x47E0_9066,
        process_table_crc32: 0x2453_EB18,
        video_crc32: 0x9AF2_0182,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x02B5_2E6D,
        process_table_crc32: 0xB415_761A,
        video_crc32: 0x75B8_EFBD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEF7F_59EE,
        process_table_crc32: 0x22C4_AD85,
        video_crc32: 0xA9A4_EDAA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEE43_8CD9,
        process_table_crc32: 0xD4FC_5E0A,
        video_crc32: 0xC918_0D5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xAB4A_95EF,
        process_table_crc32: 0xF27A_DAF4,
        video_crc32: 0x565C_5683,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x64E8_F0CF,
        process_table_crc32: 0xBAC0_AB71,
        video_crc32: 0x6750_065B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8922_874C,
        process_table_crc32: 0xA5EE_83CD,
        video_crc32: 0x734F_4830,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCC77_3947,
        process_table_crc32: 0x61C3_8BF2,
        video_crc32: 0x78DC_FBC9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x8B5B_2D22,
        process_table_crc32: 0x7969_E9FB,
        video_crc32: 0xA18E_4656,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xCE0E_9329,
        process_table_crc32: 0xA869_9A7E,
        video_crc32: 0xE25E_0683,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x23C4_E4AA,
        process_table_crc32: 0x126F_6AE3,
        video_crc32: 0xF24E_B935,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xEC66_818A,
        process_table_crc32: 0x898B_2AA4,
        video_crc32: 0xA7ED_A5F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA96F_98BC,
        process_table_crc32: 0xD13A_87AF,
        video_crc32: 0x49BA_C224,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0xA853_4D8B,
        process_table_crc32: 0x8089_7A9A,
        video_crc32: 0xDE04_7A26,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x0BB6_714A,
        process_table_crc32: 0x8ABC_E610,
        video_crc32: 0xD74C_C62E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x02E3_739D,
        video_crc32: 0x182B_5A5D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x8F93_FDF1,
        video_crc32: 0xAEB9_5B76,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xF787_F7C3,
        video_crc32: 0x0509_BC08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x14BE_E4AD,
        video_crc32: 0x0DF6_E201,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x2444_EC44,
        video_crc32: 0x4970_5E9D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xE2D8_AD1B,
        video_crc32: 0x754D_FA16,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x4B19_3C6E,
        video_crc32: 0x47E5_2E70,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xBE1B_FC8B,
        video_crc32: 0xAA36_C92D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xA827_6248,
        video_crc32: 0xEFB3_68AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x1514_E2E0,
        video_crc32: 0xB5B6_06BE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xF5B7_5CF1,
        video_crc32: 0xC4E3_B932,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x5882_7C10,
        video_crc32: 0x13AE_DA70,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x654D_EF21,
        video_crc32: 0xAC5E_358D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x9375_1CAE,
        video_crc32: 0x258F_27B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x2637_E0FA,
        video_crc32: 0xD1EB_EBD8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x0817_706A,
        video_crc32: 0x878C_37B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xBE31_BB38,
        video_crc32: 0xFF12_CE79,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x927F_87FC,
        video_crc32: 0x70A8_17E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xE473_0656,
        video_crc32: 0x5504_71A9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x4DB2_9723,
        video_crc32: 0x9B3A_D97B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x1D19_C919,
        video_crc32: 0xD5B7_5945,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xBF85_0C99,
        video_crc32: 0x707D_DB38,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x55A0_6BB1,
        video_crc32: 0xD763_C8F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x9F60_499B,
        video_crc32: 0xD01E_3EE2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x98B9_E84F,
        video_crc32: 0x5C71_475A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x561B_79B7,
        video_crc32: 0x910E_944E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xE7FC_38C6,
        video_crc32: 0x8C70_D609,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x5ACF_B86E,
        video_crc32: 0x8F0F_8B23,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x88F9_35DC,
        video_crc32: 0xCEF9_97ED,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x1759_269E,
        video_crc32: 0xA10B_8A63,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x27A3_2E77,
        video_crc32: 0x9E62_0B25,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xE13F_6F28,
        video_crc32: 0xCF89_8A49,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x48FE_FE5D,
        video_crc32: 0x4FA8_D497,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x47CC_2AE4,
        video_crc32: 0x3A37_AAEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x334C_2AE9,
        video_crc32: 0x0D53_443C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x32F1_E39A,
        video_crc32: 0x3A3D_BC44,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x3CE9_6802,
        video_crc32: 0x8437_399A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x3B30_C9D6,
        video_crc32: 0xDBD7_7AE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x06FF_5AE7,
        video_crc32: 0x810A_86A9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xF0C7_A968,
        video_crc32: 0x9E68_DD02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x9F61_A0E3,
        video_crc32: 0x3700_BD74,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xAD02_1EED,
        video_crc32: 0x84E2_6896,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x4E3B_0D83,
        video_crc32: 0xE5E7_7382,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x5FDE_3294,
        video_crc32: 0x5FFB_1B04,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0xE2ED_B23C,
        video_crc32: 0x3476_E971,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x8013_224B,
        video_crc32: 0x7D6D_08D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x8013_224B,
        video_crc32: 0xDA82_69A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x57BA_FA5C,
        video_crc32: 0xE0B1_FA03,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x6740_F2B5,
        video_crc32: 0xEC47_1A42,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x37BD_8AE4,
        video_crc32: 0x1D02_9AD5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x595A_8EF4,
        process_table_crc32: 0x1CD7_B85A,
        video_crc32: 0xCB7F_D77E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0207_953E,
        video_crc32: 0xA5A4_2AD1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF10E_0715,
        video_crc32: 0x66D6_7EC2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD5DC_5F22,
        video_crc32: 0xDE79_0A1A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC2CD_580A,
        video_crc32: 0x44B5_3CAD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x74EB_9358,
        video_crc32: 0x10B7_C1A3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4411_9BB1,
        video_crc32: 0x227F_AB56,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x828D_DAEE,
        video_crc32: 0xECE3_7B14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2B4C_4B9B,
        video_crc32: 0x41A8_4762,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE2D9_4463,
        video_crc32: 0x66B8_F6DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF4E5_DAA0,
        video_crc32: 0x8721_A60D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6609_E9B0,
        video_crc32: 0xEBED_952F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6811_6228,
        video_crc32: 0x016A_A486,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6FC8_C3FC,
        video_crc32: 0xDA77_ACB7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5207_50CD,
        video_crc32: 0xC1AD_7099,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA43F_A342,
        video_crc32: 0x49F7_7D0B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEE07_E964,
        video_crc32: 0x0B74_DD5C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC56D_DBDA,
        video_crc32: 0x435E_F4AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2FBA_BB5D,
        video_crc32: 0x1976_AC04,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0A32_0955,
        video_crc32: 0x26BA_2966,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2F30_056D,
        video_crc32: 0x4621_DAA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCCF6_953A,
        video_crc32: 0x5427_CCE0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFFBF_7415,
        video_crc32: 0xFE97_E736,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD195_5198,
        video_crc32: 0xE6B2_1BCE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF5F8_F7D3,
        video_crc32: 0x317C_7D1C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8D9C_48FF,
        video_crc32: 0x7B7E_D8AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA721_E884,
        video_crc32: 0x7436_83BA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x44E7_78D3,
        video_crc32: 0x28E2_E4B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC997_F6BF,
        video_crc32: 0x28E2_E4B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF59C_7637,
        video_crc32: 0x894E_F0F6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB183_FC8D,
        video_crc32: 0x1167_DE3D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8CB3_EC85,
        video_crc32: 0x1167_DE3D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6240_E70A,
        video_crc32: 0xCC7E_E803,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEEDB_A777,
        video_crc32: 0x181D_B64C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0D1D_3720,
        video_crc32: 0x2852_3ADB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF600_D8DD,
        video_crc32: 0x48CD_23F5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD388_6AD5,
        video_crc32: 0xCA75_736D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x60EB_5FE3,
        video_crc32: 0x46D2_B5E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3DFD_59E9,
        video_crc32: 0x06B9_461F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFD24_77EB,
        video_crc32: 0x95B8_4C3A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1EE2_E7BC,
        video_crc32: 0xDFAB_65CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE8DA_1433,
        video_crc32: 0x2762_B0AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x12D5_4B6B,
        video_crc32: 0x7C27_F054,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x73B8_78F7,
        video_crc32: 0xFFC8_6DCC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE8FA_B20A,
        video_crc32: 0xA5BD_265C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD47B_8CB2,
        video_crc32: 0xD1A7_259F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE870_0C3A,
        video_crc32: 0x6E69_D901,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0BB6_9C6D,
        video_crc32: 0x6F7D_247A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2788_04B1,
        video_crc32: 0x4EB4_E358,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC97B_0F3E,
        video_crc32: 0x0B17_86A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA100_FDF9,
        video_crc32: 0x4EB4_E358,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD964_42D5,
        video_crc32: 0xFBD6_0AD3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB524_E6DB,
        video_crc32: 0xB9AB_EB00,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x56E2_768C,
        video_crc32: 0xA83E_B63D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC207_3BC5,
        video_crc32: 0x42B2_82D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9F11_3DCF,
        video_crc32: 0x4FC0_9516,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC9A9_2AC3,
        video_crc32: 0x4FC0_9516,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x275A_214C,
        video_crc32: 0x6436_9338,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xABC1_6131,
        video_crc32: 0x7DA0_64D7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4807_F166,
        video_crc32: 0x9053_7E76,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7481_0914,
        video_crc32: 0x9540_657B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5109_BB1C,
        video_crc32: 0xA25A_4389,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDCFD_5112,
        video_crc32: 0xD364_AA18,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x98E2_DBA8,
        video_crc32: 0x6369_E7A0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4132_791A,
        video_crc32: 0x2635_8FCD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA2F4_E94D,
        video_crc32: 0xA95D_26CD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x54CC_1AC2,
        video_crc32: 0x77ED_39AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x716D_126B,
        video_crc32: 0x69E1_32C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0909_AD47,
        video_crc32: 0x77EE_86C8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFAFF_BC55,
        video_crc32: 0xAFD7_3819,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC67E_82ED,
        video_crc32: 0xAF4F_0D4D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFA75_0265,
        video_crc32: 0xB4EE_1171,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x19B3_9232,
        video_crc32: 0x5351_43E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1013_4943,
        video_crc32: 0xEE95_3549,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFEE0_42CC,
        video_crc32: 0xEE7C_A83A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE41A_3BBF,
        video_crc32: 0x59D8_7F72,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8577_0823,
        video_crc32: 0x4813_EDBA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB6C3_24E8,
        video_crc32: 0xB560_FFB1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5505_B4BF,
        video_crc32: 0x344D_B836,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1ED2_E192,
        video_crc32: 0xC944_9AD9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3BD0_EDAA,
        video_crc32: 0xC944_9AD9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x66C6_EBA0,
        video_crc32: 0x8C0E_625A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0EE9_2394,
        video_crc32: 0xC0BA_916B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE01A_281B,
        video_crc32: 0x0E5A_CA6E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6C81_6866,
        video_crc32: 0x876B_F397,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8F47_F831,
        video_crc32: 0x1B44_D330,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDC3E_5C8B,
        video_crc32: 0x2B0B_5FA7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE0BF_6233,
        video_crc32: 0x521C_572E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3854_5001,
        video_crc32: 0xAA3B_1B81,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7C4B_DABB,
        video_crc32: 0xF23D_D549,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA59B_7809,
        video_crc32: 0x1317_FC01,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x465D_E85E,
        video_crc32: 0x5BF4_4C77,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFF17_F3D9,
        video_crc32: 0x28A9_77B3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD137_6349,
        video_crc32: 0x4D74_0572,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x74E2_768C,
        video_crc32: 0x70A0_444F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x516A_C484,
        video_crc32: 0x42E8_F3AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7468_C8BC,
        video_crc32: 0x42E8_F3AB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x97AE_58EB,
        video_crc32: 0x3E35_A8D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x643E_97C6,
        video_crc32: 0x5312_E936,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8ACD_9C49,
        video_crc32: 0x82D8_D3CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAEA0_3A02,
        video_crc32: 0x1F05_CC26,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD6C4_852E,
        video_crc32: 0x8C1A_241E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC1D2_2686,
        video_crc32: 0xC55B_BB2E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2214_B6D1,
        video_crc32: 0xE335_7A2F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x936F_B835,
        video_crc32: 0x26A2_1521,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD770_328F,
        video_crc32: 0x86CC_2548,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEA40_2287,
        video_crc32: 0x949C_4284,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x04B3_2908,
        video_crc32: 0x100E_8740,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8828_6975,
        video_crc32: 0x367B_E15D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6BEE_F922,
        video_crc32: 0x2DF6_F224,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAD58_150C,
        video_crc32: 0x468C_7B28,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x88D0_A704,
        video_crc32: 0x80D6_9470,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3BB3_9232,
        video_crc32: 0x80D6_9470,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x66A5_9438,
        video_crc32: 0x48BD_703F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA67C_BA3A,
        video_crc32: 0x48BD_703F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x45BA_2A6D,
        video_crc32: 0x66AE_61BD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB382_D9E2,
        video_crc32: 0xB0DD_11F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x498D_86BA,
        video_crc32: 0xF35F_A42C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x28E0_B526,
        video_crc32: 0xC7A1_4AA6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF55F_7BAE,
        video_crc32: 0x7B33_BF3F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC9DE_4516,
        video_crc32: 0x02C0_A2E3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF5D5_C59E,
        video_crc32: 0xDBCD_B570,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1613_55C9,
        video_crc32: 0x255F_417F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3A2D_CD15,
        video_crc32: 0x9273_7766,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD4DE_C69A,
        video_crc32: 0x7F6B_0547,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBCA5_345D,
        video_crc32: 0x3AC8_60B7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC4C1_8B71,
        video_crc32: 0xA942_6F5A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEE7C_2B0A,
        video_crc32: 0xA979_993C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0DBA_BB5D,
        video_crc32: 0x515E_D593,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBC5D_FA2C,
        video_crc32: 0xAD3B_CC08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x016E_7A84,
        video_crc32: 0xAD3B_CC08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC449_F01E,
        video_crc32: 0xAD3B_CC08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x92F1_E712,
        video_crc32: 0x684D_BB17,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7C02_EC9D,
        video_crc32: 0x2E58_3DDA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF099_ACE0,
        video_crc32: 0x684D_BB17,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x135F_3CB7,
        video_crc32: 0x28EB_E979,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1272_C716,
        video_crc32: 0x7F09_2D1C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x37FA_751E,
        video_crc32: 0x03D4_7666,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBA0E_9F10,
        video_crc32: 0x03D4_7666,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFE11_15AA,
        video_crc32: 0x6D9C_C218,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x27C1_B718,
        video_crc32: 0x438F_D39A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC407_274F,
        video_crc32: 0x40F0_8EB0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x323F_D4C0,
        video_crc32: 0x9DA2_5128,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x179E_DC69,
        video_crc32: 0x1101_8CA0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6FFA_6345,
        video_crc32: 0xB67B_DAA3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA1A7_7184,
        video_crc32: 0xB3FA_3DDD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9D26_4F3C,
        video_crc32: 0xCEE9_D79B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA12D_CFB4,
        video_crc32: 0xD8D9_7848,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x42EB_5FE3,
        video_crc32: 0xC6A5_BF3F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4B4B_8492,
        video_crc32: 0xFCF4_4F4A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA5B8_8F1D,
        video_crc32: 0x6FEB_339D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBF42_F66E,
        video_crc32: 0xFB42_8C7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDE2F_C5F2,
        video_crc32: 0x4923_84D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6BE3_FAD9,
        video_crc32: 0x0109_AD22,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8825_6A8E,
        video_crc32: 0x01AA_6E10,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE6F0_339B,
        video_crc32: 0xB64A_81C7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBBE6_3591,
        video_crc32: 0xD705_32C1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD3C9_FDA5,
        video_crc32: 0xF86F_EB76,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3D3A_F62A,
        video_crc32: 0x80D1_2FC9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB1A1_B657,
        video_crc32: 0xCCDA_0D3C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5267_2600,
        video_crc32: 0x14A4_7726,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB14F_8983,
        video_crc32: 0x14A4_7726,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8DCE_B73B,
        video_crc32: 0x0AD8_B051,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5525_8509,
        video_crc32: 0xC0A1_C4DF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x113A_0FB3,
        video_crc32: 0x888B_ED29,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC8EA_AD01,
        video_crc32: 0xF934_87F0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2B2C_3D56,
        video_crc32: 0xB3F8_7514,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDD14_CED9,
        video_crc32: 0xAAC6_0C66,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBC46_B641,
        video_crc32: 0x69A1_F347,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2438_A057,
        video_crc32: 0x7F91_5C94,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x01B0_125F,
        video_crc32: 0xAF37_D4CA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x24B2_1E67,
        video_crc32: 0xD5E3_68E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC774_8E30,
        video_crc32: 0x7B57_96AF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x34E4_411D,
        video_crc32: 0x5544_872D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDA17_4A92,
        video_crc32: 0xE87E_41CF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFE7A_ECD9,
        video_crc32: 0x7773_D9B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x861E_53F5,
        video_crc32: 0x8298_B653,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xACA3_F38E,
        video_crc32: 0x5613_C906,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4F65_63D9,
        video_crc32: 0x0CF9_F055,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC215_EDB5,
        video_crc32: 0xE4CA_1584,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFE1E_6D3D,
        video_crc32: 0x3908_2B08,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBA01_E787,
        video_crc32: 0x5ACD_A472,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8731_F78F,
        video_crc32: 0xF2AF_2F14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x69C2_FC00,
        video_crc32: 0xFCF3_FD23,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE559_BC7D,
        video_crc32: 0x6BB3_F6F7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x069F_2C2A,
        video_crc32: 0xF67E_C9C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x86D4_C471,
        video_crc32: 0xCAC1_DAFD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA35C_7679,
        video_crc32: 0xB843_FEE1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x103F_434F,
        video_crc32: 0x1047_9EF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4D29_4545,
        video_crc32: 0x2CF8_8DCB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8DF0_6B47,
        video_crc32: 0x873E_8DF5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6E36_FB10,
        video_crc32: 0x1A73_83EA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x980E_089F,
        video_crc32: 0x3AC5_F5D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6201_57C7,
        video_crc32: 0xF595_CB14,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x036C_645B,
        video_crc32: 0xAC87_D62D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x982E_AEA6,
        video_crc32: 0x91FA_C5B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA4AF_901E,
        video_crc32: 0x0CB7_CBAD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x98A4_1096,
        video_crc32: 0x9537_9332,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7B62_80C1,
        video_crc32: 0x68A8_2476,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x575C_181D,
        video_crc32: 0x825B_5346,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB9AF_1392,
        video_crc32: 0xEC01_17D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD1D4_E155,
        video_crc32: 0xD9B5_3257,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA9B0_5E79,
        video_crc32: 0xD9B5_3257,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBEA6_FDD1,
        video_crc32: 0xD680_962C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5D60_6D86,
        video_crc32: 0x9EAA_BFDA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC985_20CF,
        video_crc32: 0x7BFE_4A63,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9493_26C5,
        video_crc32: 0x2FA2_EC21,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC22B_31C9,
        video_crc32: 0x41EA_585F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2CD8_3A46,
        video_crc32: 0xF22C_0B6F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA043_7A3B,
        video_crc32: 0x9446_7426,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4385_EA6C,
        video_crc32: 0xB50A_8967,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7F03_121E,
        video_crc32: 0x49EF_63A1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5A8B_A016,
        video_crc32: 0x1E95_9290,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD77F_4A18,
        video_crc32: 0x1E95_9290,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9360_C0A2,
        video_crc32: 0x1E95_9290,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4AB0_6210,
        video_crc32: 0xAD02_CFC2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA976_F247,
        video_crc32: 0x7CDA_E0AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5F4E_01C8,
        video_crc32: 0xA6AD_E306,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7AEF_0961,
        video_crc32: 0x73E9_ED33,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x028B_B64D,
        video_crc32: 0x339E_13C0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7C87_AFB5,
        video_crc32: 0x426F_FC20,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4006_910D,
        video_crc32: 0xFDA1_00BE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7C0D_1185,
        video_crc32: 0x9AE1_F0FF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9FCB_81D2,
        video_crc32: 0x4396_C4B0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x966B_5AA3,
        video_crc32: 0x5A46_200D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7898_512C,
        video_crc32: 0x6066_14AA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6262_285F,
        video_crc32: 0x1935_A818,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x030F_1BC3,
        video_crc32: 0xF329_06B0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x30BB_3708,
        video_crc32: 0x6AA9_5E2F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD37D_A75F,
        video_crc32: 0x74D1_328E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x98AA_F272,
        video_crc32: 0x4BCD_3A0C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBDA8_FE4A,
        video_crc32: 0x566B_E143,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE0BE_F840,
        video_crc32: 0x0237_4701,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8891_3074,
        video_crc32: 0xFEE9_116C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6662_3BFB,
        video_crc32: 0xA0C0_A383,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEAF9_7B86,
        video_crc32: 0x6268_4F11,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x093F_EBD1,
        video_crc32: 0xDB2A_F4CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD7BC_4781,
        video_crc32: 0x9C07_9585,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEB3D_7939,
        video_crc32: 0xCCB5_2B66,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x33D6_4B0B,
        video_crc32: 0xC275_E517,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x77C9_C1B1,
        video_crc32: 0xFBA0_ACA5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAE19_6303,
        video_crc32: 0x0997_B94F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4DDF_F354,
        video_crc32: 0x12B5_BAAC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBBE7_00DB,
        video_crc32: 0x7D0B_9725,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBBD8_4BDF,
        video_crc32: 0x519E_ACF4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDAB5_7843,
        video_crc32: 0xF7F5_1F5C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7F60_6D86,
        video_crc32: 0x2AB6_0926,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5AE8_DF8E,
        video_crc32: 0x3366_ED9B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7FEA_D3B6,
        video_crc32: 0x5DC7_66A6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9C2C_43E1,
        video_crc32: 0xE2B0_472C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6FBC_8CCC,
        video_crc32: 0xB0B1_60A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x814F_8743,
        video_crc32: 0xD5C6_C36D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA522_2108,
        video_crc32: 0xFE2E_5279,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDD46_9E24,
        video_crc32: 0x16A5_C859,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB106_3A2A,
        video_crc32: 0x2753_33F9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x52C0_AA7D,
        video_crc32: 0xEF6D_6E20,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE3BB_A499,
        video_crc32: 0x58E6_54C6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA7A4_2E23,
        video_crc32: 0x656D_118E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9A94_3E2B,
        video_crc32: 0x42D0_AA5E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7467_35A4,
        video_crc32: 0xB7F5_6357,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF8FC_75D9,
        video_crc32: 0x2A92_743C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1B3A_E58E,
        video_crc32: 0xFE98_9D1A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDD8C_09A0,
        video_crc32: 0x3C96_8BBF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF804_BBA8,
        video_crc32: 0xDCF7_61A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4B67_8E9E,
        video_crc32: 0xBB6E_BF81,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1671_8894,
        video_crc32: 0x3A43_F806,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD6A8_A696,
        video_crc32: 0xB457_5482,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x356E_36C1,
        video_crc32: 0x7FC3_BF75,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7614_391A,
        video_crc32: 0x0D2F_380B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5834_A98A,
        video_crc32: 0xCD88_0BE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFEDD_60A4,
        video_crc32: 0xACC7_B8E1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC25C_5E1C,
        video_crc32: 0xDA21_CD6E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFE57_DE94,
        video_crc32: 0xF046_ECD4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1D91_4EC3,
        video_crc32: 0xF046_ECD4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x31AF_D61F,
        video_crc32: 0x22EC_0DB3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDF5C_DD90,
        video_crc32: 0x0818_D0C4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB727_2F57,
        video_crc32: 0x3B20_9745,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCF43_907B,
        video_crc32: 0xCECB_F8A4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE5FE_3000,
        video_crc32: 0xB483_5BB8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0638_A057,
        video_crc32: 0xD4A7_95B6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB7DF_E126,
        video_crc32: 0x696C_E42E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x92DD_ED1E,
        video_crc32: 0x98D4_AAB8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCFCB_EB14,
        video_crc32: 0x531A_8CDA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9973_FC18,
        video_crc32: 0xB66C_195C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7780_F797,
        video_crc32: 0x0743_6426,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFB1B_B7EA,
        video_crc32: 0xB61C_9126,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x46CD_C52A,
        video_crc32: 0xE24F_BDCE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x572F_3CF7,
        video_crc32: 0xEB1A_20EE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x38A0_8FDD,
        video_crc32: 0xF10A_B726,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFF53_64F1,
        video_crc32: 0xC072_243B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6545_ED2D,
        video_crc32: 0xC0B3_E2B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x629C_4CF9,
        video_crc32: 0x434E_D326,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCB5D_DD8C,
        video_crc32: 0xAB3B_E3A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3D65_2E03,
        video_crc32: 0xE875_58B8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x52C3_2788,
        video_crc32: 0x2805_0DD0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x07C3_990B,
        video_crc32: 0x5D66_74D8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE4FA_8A65,
        video_crc32: 0x0331_DA36,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x927C_B5FF,
        video_crc32: 0x454A_0DEC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE470_3455,
        video_crc32: 0x1933_7B99,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD9BF_A764,
        video_crc32: 0x4645_2F7A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0E16_7F73,
        video_crc32: 0x5839_E80D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAAE2_75DE,
        video_crc32: 0x143F_3682,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFA1F_0D8F,
        video_crc32: 0xB935_65F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8BBD_3C6F,
        video_crc32: 0x6D36_6F83,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x956D_110B,
        video_crc32: 0x007D_79A0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3CAC_807E,
        video_crc32: 0xC238_498C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x187E_D849,
        video_crc32: 0x3FAA_D928,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9B61_DD25,
        video_crc32: 0xDFD8_32BB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2D47_1677,
        video_crc32: 0xCDB5_552D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x89B3_1CDA,
        video_crc32: 0x8816_30DD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4F2F_5D85,
        video_crc32: 0xCDB5_552D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x818D_CC7D,
        video_crc32: 0xE7AA_EF31,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4FC1_6251,
        video_crc32: 0xC8EF_3532,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3947_5DCB,
        video_crc32: 0x3E0B_D8A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xABAB_6EDB,
        video_crc32: 0x5A2F_375E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x31BD_E707,
        video_crc32: 0x3303_7F02,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3664_46D3,
        video_crc32: 0x2C8B_7672,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9FA5_D7A6,
        video_crc32: 0x5653_1172,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x699D_2429,
        video_crc32: 0x7AA7_D347,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x23A5_6E0F,
        video_crc32: 0x8AD6_EEFC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2951_5849,
        video_crc32: 0x52A8_94E6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA1E0_4C23,
        video_crc32: 0x0BA5_69F8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCE6F_FF09,
        video_crc32: 0x4EB4_023E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA16A_F213,
        video_crc32: 0x1D92_00D1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9CA5_6122,
        video_crc32: 0x0012_0AE7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB13C_AD69,
        video_crc32: 0xB4B7_84E6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x15C8_A7C4,
        video_crc32: 0xCF33_9945,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7BA2_00AD,
        video_crc32: 0xDD7D_A441,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2EA2_BE2E,
        video_crc32: 0x233E_DD5C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x297B_1FFA,
        video_crc32: 0x6B27_6C9F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x80BA_8E8F,
        video_crc32: 0x1A30_4242,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0DCA_00E3,
        video_crc32: 0x011E_9E8D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7BC6_8149,
        video_crc32: 0x011E_9E8D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE1D0_0895,
        video_crc32: 0x4704_408F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x02E9_1BFB,
        video_crc32: 0x2970_C453,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA61D_1156,
        video_crc32: 0x813C_6336,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6081_5009,
        video_crc32: 0xAC0A_0513,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9388_C222,
        video_crc32: 0x2F07_2AFB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x785A_2FA3,
        video_crc32: 0x2F07_2AFB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x17D5_9C89,
        video_crc32: 0x02E4_5E1E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEEB1_A89D,
        video_crc32: 0x0EA8_5EA0,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6DAE_ADF1,
        video_crc32: 0xE6EC_39D4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x737E_8095,
        video_crc32: 0x1E22_B679,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDABF_11E0,
        video_crc32: 0xBA24_CFA7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2C87_E26F,
        video_crc32: 0xFD34_6838,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD086_8E26,
        video_crc32: 0x7AAB_5A19,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x66A0_4574,
        video_crc32: 0x7AAB_5A19,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1026_7AEE,
        video_crc32: 0x4A0C_839A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x662A_FB44,
        video_crc32: 0xA82C_73A3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5BE5_6875,
        video_crc32: 0xD264_D0BF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA9D2_F3CF,
        video_crc32: 0x278F_BF5E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0D26_F962,
        video_crc32: 0xE389_255F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2F5A_0A87,
        video_crc32: 0x0ECA_DEAF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCA0B_BF3D,
        video_crc32: 0x0ECA_DEAF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCDD2_1EE9,
        video_crc32: 0xF17D_2889,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6413_8F9C,
        video_crc32: 0xBCE4_1B7D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBAF1_C3F7,
        video_crc32: 0x0C0B_D8E6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x39EE_C69B,
        video_crc32: 0x80C2_4250,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB15F_D2F1,
        video_crc32: 0x824E_8C9F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x15AB_D85C,
        video_crc32: 0x7540_BF6C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD337_9903,
        video_crc32: 0xA03E_A27F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1D95_08FB,
        video_crc32: 0x4598_9367,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0C77_F126,
        video_crc32: 0xD151_E18B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x63F8_420C,
        video_crc32: 0x7F90_A59C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA40B_A920,
        video_crc32: 0x726D_D134,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3E1D_20FC,
        video_crc32: 0xF902_1A36,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x39C4_8128,
        video_crc32: 0xA20E_ADF7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9005_105D,
        video_crc32: 0xA235_5B91,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x663D_E3D2,
        video_crc32: 0x109B_5D19,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x099B_EA59,
        video_crc32: 0x14DF_DAB9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6130_5709,
        video_crc32: 0x4ABB_9D4A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8209_4467,
        video_crc32: 0x46A4_409F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF48F_7BFD,
        video_crc32: 0xB7A1_9453,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8283_FA57,
        video_crc32: 0xB7ED_8EF9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBF4C_6966,
        video_crc32: 0x60C2_9D18,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x68E5_B171,
        video_crc32: 0xC152_2A3C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCC11_BBDC,
        video_crc32: 0xE7DE_A159,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9CEC_C38D,
        video_crc32: 0xDBE0_3F6B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD0E5_F1BE,
        video_crc32: 0xB82C_BB24,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCE35_DCDA,
        video_crc32: 0x624A_4A3B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x67F4_4DAF,
        video_crc32: 0x624A_4A3B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2C23_1882,
        video_crc32: 0x2A14_87AD,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4326_1598,
        video_crc32: 0x296B_DA87,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC039_10F4,
        video_crc32: 0x2CEA_3DF9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x761F_DBA6,
        video_crc32: 0x64C0_140F,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD2EB_D10B,
        video_crc32: 0x7A54_3058,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1477_9054,
        video_crc32: 0xFFCB_1CE5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9C28_05D9,
        video_crc32: 0xFFCB_1CE5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5264_ABF5,
        video_crc32: 0x9F36_BFC1,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x24E2_946F,
        video_crc32: 0xF9EA_B8CE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB60E_A77F,
        video_crc32: 0x92A9_6629,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2C18_2EA3,
        video_crc32: 0x92A9_6629,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2BC1_8F77,
        video_crc32: 0x8B88_7A98,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8200_1E02,
        video_crc32: 0x88F7_27B2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7438_ED8D,
        video_crc32: 0xBD78_03A5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3E00_A7AB,
        video_crc32: 0xA8A4_0E85,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x7209_9598,
        video_crc32: 0x6378_F57A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFAB8_81F2,
        video_crc32: 0x65F2_EC8B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9537_32D8,
        video_crc32: 0x032E_EB84,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFA32_3FC2,
        video_crc32: 0xD35B_4D44,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC7FD_ACF3,
        video_crc32: 0x0584_C813,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEA64_60B8,
        video_crc32: 0xB7A7_50F7,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4E90_6A15,
        video_crc32: 0xD98A_F8A9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x20FA_CD7C,
        video_crc32: 0xF794_5436,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4851_702C,
        video_crc32: 0x4F21_862B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4F88_D1F8,
        video_crc32: 0xA76B_3D1B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE649_408D,
        video_crc32: 0x40FE_1153,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1D35_4F4B,
        video_crc32: 0x40FE_1153,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x8723_C697,
        video_crc32: 0x0336_1C78,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x641A_D5F9,
        video_crc32: 0x9E52_709A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC0EE_DF54,
        video_crc32: 0x3299_2BDA,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0672_9E0B,
        video_crc32: 0xF413_20A3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC8D0_0FF3,
        video_crc32: 0xA579_B699,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2302_E272,
        video_crc32: 0xEC2E_2F6C,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4C8D_5158,
        video_crc32: 0x022E_647B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB5E9_654C,
        video_crc32: 0x8B3A_D6CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x36F6_6020,
        video_crc32: 0xB33B_76CB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x2826_4D44,
        video_crc32: 0xFB39_96C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x81E7_DC31,
        video_crc32: 0xF846_CBE8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x77DF_2FBE,
        video_crc32: 0xAF9D_9603,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC7D7_71C4,
        video_crc32: 0x4F18_D0E6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0DA6_5017,
        video_crc32: 0x4078_5019,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBB80_9B45,
        video_crc32: 0xF64C_8024,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xCD06_A4DF,
        video_crc32: 0xF937_813E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBB0A_2575,
        video_crc32: 0x70F1_5B53,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x86C5_B644,
        video_crc32: 0xF301_D02D,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x74F2_2DFE,
        video_crc32: 0x69CD_E69A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD006_2753,
        video_crc32: 0xB8D2_09C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF27A_D4B6,
        video_crc32: 0x8E3C_E486,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA77A_6A35,
        video_crc32: 0x1AC3_512A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA0A3_CBE1,
        video_crc32: 0x2F1F_6C7B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0962_5A94,
        video_crc32: 0xB996_F9F6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xB885_1BE5,
        video_crc32: 0x2993_39E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD780_16FF,
        video_crc32: 0x2993_39E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x549F_1393,
        video_crc32: 0x2993_39E4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDC2E_07F9,
        video_crc32: 0x2173_CD8B,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x78DA_0D54,
        video_crc32: 0x8F00_59AE,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xBE46_4C0B,
        video_crc32: 0x510C_8FEB,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4D4F_DE20,
        video_crc32: 0x12EE_8E9E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5CAD_27FD,
        video_crc32: 0x39D1_1181,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3322_94D7,
        video_crc32: 0x7FCB_CF83,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF4D1_7FFB,
        video_crc32: 0xC7BE_9186,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x6EC7_F627,
        video_crc32: 0x60E9_4364,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x691E_57F3,
        video_crc32: 0x8065_51F4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC0DF_C686,
        video_crc32: 0x969E_5B68,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4502_121C,
        video_crc32: 0x0CBD_B447,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0C41_8201,
        video_crc32: 0x1A92_9125,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEF78_916F,
        video_crc32: 0x30E0_6253,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x99FE_AEF5,
        video_crc32: 0x2ACD_43CC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEFF2_2F5F,
        video_crc32: 0x11C0_5C68,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xD23D_BC6E,
        video_crc32: 0x118C_46C2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x0594_6479,
        video_crc32: 0x7F5A_F9A8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xA160_6ED4,
        video_crc32: 0x8EE4_80C5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF19D_1685,
        video_crc32: 0x2815_90BC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xFB69_20C3,
        video_crc32: 0xABEB_D4E6,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xE5B9_0DA7,
        video_crc32: 0xAD54_5C87,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4C78_9CD2,
        video_crc32: 0xD71E_FC0A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x68AA_C4E5,
        video_crc32: 0xD2F2_CED3,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEBB5_C189,
        video_crc32: 0xD2C9_38B5,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5D93_0ADB,
        video_crc32: 0xC36B_11AC,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF967_0076,
        video_crc32: 0x7D54_6C07,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3FFB_4129,
        video_crc32: 0xB060_CAD4,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xF159_D0D1,
        video_crc32: 0x7A4F_A8D2,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x3F15_7EFD,
        video_crc32: 0xDE9A_F85A,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4993_4167,
        video_crc32: 0x43FD_DCCF,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xDB7F_7277,
        video_crc32: 0x2C09_7E06,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x4169_FBAB,
        video_crc32: 0x7BEB_BA63,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x46B0_5A7F,
        video_crc32: 0xBC5D_05B9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xEF71_CB0A,
        video_crc32: 0x7E5F_A34E,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x1949_3885,
        video_crc32: 0x1A44_56C9,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x5371_72A3,
        video_crc32: 0x9A2A_B0E8,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x22D3_4343,
        video_crc32: 0x268B_DF11,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAA62_5729,
        video_crc32: 0xF5D2_F184,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xC5ED_E403,
        video_crc32: 0xF879_8784,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0xAAE8_E919,
        video_crc32: 0x0632_1103,
    },
    RedLabelTraceCrcSample {
        object_table_crc32: 0x65B1_5F89,
        process_table_crc32: 0x9727_7A28,
        video_crc32: 0xB466_EC8C,
    },
];

pub(crate) const RED_LABEL_LONG_INSTRUCTION_RAND_FIRST_FRAME: u64 = 1895;
pub(crate) const RED_LABEL_LONG_INSTRUCTION_RAND_SAMPLES: &[RedLabelTraceRandSample] = &[
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0xDA,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0xB6,
        lseed: 0xA1,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xDB,
        lseed: 0x50,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0x6D,
        lseed: 0xA8,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0xB6,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0x5B,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0xAD,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0x1E,
        hseed: 0xD6,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0xEB,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x75,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x3A,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x1D,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0x0E,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x07,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0x03,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x01,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0x00,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x00,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0x48,
        hseed: 0x80,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0xC7,
        hseed: 0xC0,
        lseed: 0x1D,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x60,
        lseed: 0x0E,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0xB0,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0xD8,
        lseed: 0x03,
    },
    RedLabelTraceRandSample {
        seed: 0x26,
        hseed: 0xEC,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0xF6,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x7B,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0x3D,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x1E,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0x0F,
        lseed: 0x60,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x07,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0x03,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0x81,
        lseed: 0xEC,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xC0,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x60,
        lseed: 0x7B,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0x30,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x25,
        hseed: 0x18,
        lseed: 0x1E,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x8C,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x46,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0xC4,
        hseed: 0xA3,
        lseed: 0x03,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0xD1,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0xE8,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0x74,
        lseed: 0x60,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0x3A,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0x1D,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0x8E,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0xC7,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0x63,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0xF9,
        hseed: 0xB1,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0xD8,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0xEC,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x76,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0xBB,
        lseed: 0x1D,
    },
    RedLabelTraceRandSample {
        seed: 0x38,
        hseed: 0x5D,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0xAE,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0xD7,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x39,
        hseed: 0xEB,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0xF5,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0x96,
        hseed: 0xFA,
        lseed: 0xEC,
    },
    RedLabelTraceRandSample {
        seed: 0x47,
        hseed: 0xFD,
        lseed: 0x76,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0x7E,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0x0D,
        hseed: 0x3F,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x05,
        hseed: 0x1F,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x8F,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0xC7,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x63,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0xB1,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0xEA,
        hseed: 0xD8,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0x6C,
        lseed: 0x7E,
    },
    RedLabelTraceRandSample {
        seed: 0x34,
        hseed: 0xB6,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x5B,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0x2D,
        lseed: 0x8F,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x16,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x8B,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0xFD,
        hseed: 0xC5,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0xE2,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0xF1,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0xF8,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x7C,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x3E,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0x1F,
        lseed: 0x16,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x0F,
        lseed: 0x8B,
    },
    RedLabelTraceRandSample {
        seed: 0xB2,
        hseed: 0x07,
        lseed: 0xC5,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0x83,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x41,
        lseed: 0xF1,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x41,
        lseed: 0xF1,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xD0,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0xE8,
        lseed: 0x3E,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0xF4,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x7A,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x3D,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0x9E,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0xCF,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0xE7,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0x73,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0x39,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x9C,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x9C,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0xD8,
        hseed: 0xA7,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x8B,
        hseed: 0x53,
        lseed: 0x9E,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0xA9,
        lseed: 0xCF,
    },
    RedLabelTraceRandSample {
        seed: 0xCE,
        hseed: 0x54,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0xAA,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0xD5,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0x6A,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0xB5,
        lseed: 0x4E,
    },
    RedLabelTraceRandSample {
        seed: 0x1F,
        hseed: 0xDA,
        lseed: 0xA7,
    },
    RedLabelTraceRandSample {
        seed: 0xAE,
        hseed: 0xED,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0xF6,
        lseed: 0xA9,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0xF6,
        lseed: 0xA9,
    },
    RedLabelTraceRandSample {
        seed: 0x22,
        hseed: 0x3D,
        lseed: 0xAA,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x9E,
        lseed: 0xD5,
    },
    RedLabelTraceRandSample {
        seed: 0x0C,
        hseed: 0xCF,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0xD1,
        hseed: 0xE7,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0xF3,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0xF9,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0x4B,
        hseed: 0x7C,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x3E,
        lseed: 0x7B,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0x1F,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0x0F,
        lseed: 0x9E,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x87,
        lseed: 0xCF,
    },
    RedLabelTraceRandSample {
        seed: 0x16,
        hseed: 0x43,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0xA1,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0xD0,
        lseed: 0xF9,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x68,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0x10,
        hseed: 0xB4,
        lseed: 0x3E,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0xDA,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0x3B,
        hseed: 0x6D,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x36,
        lseed: 0x87,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x9B,
        lseed: 0x43,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0xCD,
        lseed: 0xA1,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xE6,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0x73,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0xCF,
        hseed: 0xB9,
        lseed: 0xB4,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0x5C,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0x4B,
        hseed: 0xAE,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x57,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0x2B,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0x15,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0x0A,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x05,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x82,
        lseed: 0xB9,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0x41,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x36,
        hseed: 0xA0,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0xD0,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0xD0,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xE8,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x74,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0xBA,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0xF3,
        hseed: 0xDD,
        lseed: 0x05,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xEE,
        lseed: 0x82,
    },
    RedLabelTraceRandSample {
        seed: 0xDA,
        hseed: 0x77,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0xBB,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0x5D,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0x2E,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x3B,
        hseed: 0x97,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0xC8,
        hseed: 0x4B,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0xEC,
        hseed: 0xA5,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0xEC,
        hseed: 0xA5,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xA9,
        lseed: 0x77,
    },
    RedLabelTraceRandSample {
        seed: 0xFA,
        hseed: 0xD4,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0xC7,
        hseed: 0x6A,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0xC9,
        hseed: 0x35,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x9A,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0xCD,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0x66,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0xB3,
        lseed: 0x52,
    },
    RedLabelTraceRandSample {
        seed: 0xBE,
        hseed: 0x59,
        lseed: 0xA9,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x2C,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x16,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0x33,
        hseed: 0x8B,
        lseed: 0x35,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0xC5,
        lseed: 0x9A,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0xE2,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0x71,
        lseed: 0x66,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x38,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x9C,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0x54,
        hseed: 0x4E,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xCA,
        hseed: 0xA7,
        lseed: 0x16,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x53,
        lseed: 0x8B,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x29,
        lseed: 0xC5,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x94,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0x4A,
        lseed: 0x71,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0x4A,
        lseed: 0x71,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xD2,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0xE9,
        lseed: 0x4E,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0xF4,
        lseed: 0xA7,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0xFA,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0xFD,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x7E,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0x3F,
        lseed: 0x4A,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x9F,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0xCF,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x67,
        lseed: 0xE9,
    },
    RedLabelTraceRandSample {
        seed: 0x1E,
        hseed: 0x33,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0x1E,
        hseed: 0x33,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0x8C,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0x46,
        lseed: 0x7E,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0xA3,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0xE0,
        hseed: 0x51,
        lseed: 0x9F,
    },
    RedLabelTraceRandSample {
        seed: 0xA9,
        hseed: 0x28,
        lseed: 0xCF,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0x14,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x8A,
        lseed: 0x33,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0xC5,
        lseed: 0x19,
    },
    RedLabelTraceRandSample {
        seed: 0x47,
        hseed: 0x62,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0xB1,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x58,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0x03,
        hseed: 0xAC,
        lseed: 0x51,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0xD6,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0xEB,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0x75,
        lseed: 0x8A,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0xBA,
        lseed: 0xC5,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0xDD,
        lseed: 0x62,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x6E,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x66,
        hseed: 0xB7,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0xCA,
        hseed: 0xDB,
        lseed: 0xAC,
    },
    RedLabelTraceRandSample {
        seed: 0x33,
        hseed: 0xED,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x0C,
        hseed: 0x76,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0x3B,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0x9D,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x05,
        hseed: 0xCE,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x67,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xB3,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0xD6,
        hseed: 0xD9,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x6C,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0x85,
        hseed: 0x36,
        lseed: 0x76,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0x1B,
        lseed: 0x3B,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x0D,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0xC0,
        hseed: 0x06,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x3B,
        hseed: 0x83,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xC1,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0xE0,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x70,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xB6,
        hseed: 0xB8,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0xAA,
        hseed: 0x5C,
        lseed: 0x1B,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0x2E,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x0C,
        hseed: 0x17,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x0B,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0x85,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0x97,
        hseed: 0xC2,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x61,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x30,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0x98,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0x98,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0xE6,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0xF3,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0x79,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xBC,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x5E,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0xAF,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0x57,
        lseed: 0x98,
    },
    RedLabelTraceRandSample {
        seed: 0x6B,
        hseed: 0xAB,
        lseed: 0xCC,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0xD5,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0x6A,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0xB5,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0x48,
        hseed: 0x5A,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0xAD,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0xD6,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x6B,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x11,
        hseed: 0xB5,
        lseed: 0xAB,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0x5A,
        lseed: 0xD5,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0xAD,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0x29,
        hseed: 0xD6,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0xD1,
        hseed: 0xEB,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0xF5,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0xD7,
        hseed: 0x7A,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0x3D,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0x3D,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0xE1,
        hseed: 0x8F,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0x29,
        hseed: 0xC7,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x63,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x31,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0x18,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x8C,
        lseed: 0x7A,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0xC6,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x63,
        lseed: 0x1E,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xB1,
        lseed: 0x8F,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x58,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0xAC,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0xAC,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0xEB,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0xF5,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0xFA,
        lseed: 0xC6,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0x7D,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0xBE,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0xDF,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0xEF,
        lseed: 0xAC,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xF7,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0x7B,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x3D,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0x9E,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0x9E,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x08,
        hseed: 0xCF,
        lseed: 0x7D,
    },
    RedLabelTraceRandSample {
        seed: 0x4E,
        hseed: 0x67,
        lseed: 0xBE,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0xB3,
        lseed: 0xDF,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0x59,
        lseed: 0xEF,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0x2C,
        lseed: 0xF7,
    },
    RedLabelTraceRandSample {
        seed: 0xE6,
        hseed: 0x96,
        lseed: 0x7B,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x4B,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x25,
        lseed: 0x9E,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x92,
        lseed: 0xCF,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x49,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0xA4,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0xD2,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0xD2,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0xB4,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0x66,
        hseed: 0x5A,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x95,
        hseed: 0x2D,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0xF9,
        hseed: 0x96,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0x91,
        hseed: 0x4B,
        lseed: 0x49,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0x25,
        lseed: 0xA4,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0x12,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0x64,
        hseed: 0x09,
        lseed: 0x69,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x04,
        lseed: 0xB4,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x02,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x81,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0xDC,
        hseed: 0x40,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0x10,
        hseed: 0x20,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x10,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0x0D,
        hseed: 0x88,
        lseed: 0x12,
    },
    RedLabelTraceRandSample {
        seed: 0x85,
        hseed: 0x44,
        lseed: 0x09,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x22,
        lseed: 0x04,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x11,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0x08,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0xC9,
        hseed: 0x84,
        lseed: 0x40,
    },
    RedLabelTraceRandSample {
        seed: 0xCE,
        hseed: 0x42,
        lseed: 0x20,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x21,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0x10,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0x10,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0x44,
        lseed: 0x22,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0x22,
        lseed: 0x11,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x91,
        lseed: 0x08,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0xC8,
        lseed: 0x84,
    },
    RedLabelTraceRandSample {
        seed: 0x07,
        hseed: 0x64,
        lseed: 0x42,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x32,
        lseed: 0x21,
    },
    RedLabelTraceRandSample {
        seed: 0x25,
        hseed: 0x99,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x55,
        hseed: 0x4C,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0xFA,
        hseed: 0xA6,
        lseed: 0x44,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0x53,
        lseed: 0x22,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0x29,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0x29,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x0D,
        hseed: 0xCA,
        lseed: 0x64,
    },
    RedLabelTraceRandSample {
        seed: 0xCF,
        hseed: 0x65,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0x32,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0x55,
        hseed: 0x19,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0x42,
        hseed: 0x8C,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0x46,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x30,
        hseed: 0xA3,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0x51,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0x28,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0xD6,
        hseed: 0x94,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0xCA,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0xCA,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0x32,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x7C,
        hseed: 0x99,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0x4C,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0x67,
        hseed: 0xA6,
        lseed: 0x51,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0xD3,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0xE9,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0x45,
        hseed: 0x74,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0xBA,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0xDD,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x6E,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0xFF,
        hseed: 0x37,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0x9B,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x1F,
        hseed: 0x4D,
        lseed: 0xD3,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0xA6,
        lseed: 0xE9,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0x53,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0x6B,
        hseed: 0x29,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0xC4,
        hseed: 0x94,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x4A,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xA5,
        lseed: 0x37,
    },
    RedLabelTraceRandSample {
        seed: 0x03,
        hseed: 0xD2,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0x69,
        lseed: 0x4D,
    },
    RedLabelTraceRandSample {
        seed: 0x5C,
        hseed: 0x34,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x92,
        hseed: 0x1A,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x8D,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x46,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x23,
        lseed: 0x4A,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x91,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0xC8,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0x64,
        lseed: 0x69,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0x32,
        lseed: 0x34,
    },
    RedLabelTraceRandSample {
        seed: 0x97,
        hseed: 0x19,
        lseed: 0x1A,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0x8C,
        lseed: 0x8D,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0x46,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0x23,
        lseed: 0x23,
    },
    RedLabelTraceRandSample {
        seed: 0x17,
        hseed: 0x91,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x17,
        hseed: 0x91,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0xE4,
        lseed: 0x64,
    },
    RedLabelTraceRandSample {
        seed: 0xE2,
        hseed: 0x72,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0x39,
        lseed: 0x19,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x1C,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0x8E,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x47,
        lseed: 0x23,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0xA3,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0xD1,
        lseed: 0xC8,
    },
    RedLabelTraceRandSample {
        seed: 0xFF,
        hseed: 0xE8,
        lseed: 0xE4,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x74,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0x3A,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0x1D,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0x8E,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0x7B,
        hseed: 0xC7,
        lseed: 0x47,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0xE3,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xF1,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0xF8,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xFC,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x7E,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0xBF,
        lseed: 0x1D,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x5F,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0xAF,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0xD7,
        lseed: 0xE3,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0xEB,
        lseed: 0xF1,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0xF5,
        lseed: 0xF8,
    },
    RedLabelTraceRandSample {
        seed: 0xE6,
        hseed: 0xFA,
        lseed: 0xFC,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0xFD,
        lseed: 0x7E,
    },
    RedLabelTraceRandSample {
        seed: 0x8C,
        hseed: 0xFE,
        lseed: 0xBF,
    },
    RedLabelTraceRandSample {
        seed: 0x94,
        hseed: 0x7F,
        lseed: 0x5F,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0x3F,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x1F,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x8F,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0x47,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0xA3,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x6B,
        hseed: 0xD1,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x68,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0xB4,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x5A,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0x2D,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x16,
        lseed: 0x8F,
    },
    RedLabelTraceRandSample {
        seed: 0xE3,
        hseed: 0x0B,
        lseed: 0x47,
    },
    RedLabelTraceRandSample {
        seed: 0xE3,
        hseed: 0x85,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0x4E,
        hseed: 0xC2,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0x45,
        hseed: 0xE1,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0x85,
        hseed: 0xF0,
        lseed: 0xB4,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x78,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0xBC,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0xBC,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0x5E,
        lseed: 0x16,
    },
    RedLabelTraceRandSample {
        seed: 0xAA,
        hseed: 0x2F,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0x17,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x8B,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x55,
        hseed: 0x45,
        lseed: 0xE1,
    },
    RedLabelTraceRandSample {
        seed: 0xA3,
        hseed: 0xA2,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0xC4,
        hseed: 0x51,
        lseed: 0x78,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0xA8,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x89,
        hseed: 0xD4,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0xEA,
        lseed: 0x2F,
    },
    RedLabelTraceRandSample {
        seed: 0xEC,
        hseed: 0x75,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0xBA,
        lseed: 0x8B,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0xBA,
        lseed: 0x8B,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0xAE,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0x57,
        lseed: 0x51,
    },
    RedLabelTraceRandSample {
        seed: 0x64,
        hseed: 0xAB,
        lseed: 0xA8,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0xD5,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x6A,
        lseed: 0xEA,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x6A,
        lseed: 0xEA,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0xDA,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0xED,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0xE4,
        hseed: 0x76,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0xBB,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0xDD,
        lseed: 0xAB,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0xDD,
        lseed: 0xAB,
    },
    RedLabelTraceRandSample {
        seed: 0x8B,
        hseed: 0xB7,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0xDB,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0xA2,
        hseed: 0xED,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0xF6,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0x94,
        hseed: 0x7B,
        lseed: 0x76,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x3D,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x1E,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0x0F,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0x87,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0xC3,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x61,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x61,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0x30,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0xB0,
        hseed: 0x0C,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x45,
        hseed: 0x06,
        lseed: 0x1E,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x83,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0x41,
        lseed: 0x87,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0x41,
        lseed: 0x87,
    },
    RedLabelTraceRandSample {
        seed: 0x48,
        hseed: 0xD0,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x02,
        hseed: 0xE8,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xA3,
        hseed: 0x74,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0xC1,
        hseed: 0xBA,
        lseed: 0x0C,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xDD,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xDD,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x6E,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0xB7,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0xDB,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0x6D,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0x36,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x89,
        hseed: 0x9B,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0x4D,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0xA6,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0x53,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0xA9,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0xD4,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0x54,
        hseed: 0x6A,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x54,
        hseed: 0x6A,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0x1A,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0x0D,
        lseed: 0x4D,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0x06,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x59,
        hseed: 0x03,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x81,
        lseed: 0xA9,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0x40,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0x20,
        lseed: 0x6A,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0x90,
        lseed: 0x35,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0xC8,
        lseed: 0x1A,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0xE4,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0x72,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0x72,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x06,
        hseed: 0x9C,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0x31,
        hseed: 0xCE,
        lseed: 0x40,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0x67,
        lseed: 0x20,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x33,
        lseed: 0x90,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x19,
        lseed: 0xC8,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0x8C,
        lseed: 0xE4,
    },
    RedLabelTraceRandSample {
        seed: 0xE3,
        hseed: 0x46,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0x16,
        hseed: 0x23,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0x11,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x67,
        hseed: 0x88,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0xC4,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0xC4,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0xF1,
        lseed: 0x19,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x78,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0xBC,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0x5E,
        lseed: 0x23,
    },
    RedLabelTraceRandSample {
        seed: 0xEA,
        hseed: 0xAF,
        lseed: 0x11,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0xD7,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0x4E,
        hseed: 0xEB,
        lseed: 0xC4,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0x75,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x3A,
        lseed: 0xF1,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x9D,
        lseed: 0x78,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xCE,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xCE,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0xE7,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x17,
        hseed: 0xF3,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x79,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0xBC,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x5E,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0xAF,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0xD7,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x6B,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x10,
        hseed: 0xB5,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0xDA,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0xA4,
        hseed: 0xED,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0x30,
        hseed: 0x76,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x30,
        hseed: 0x76,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xCB,
        hseed: 0xDD,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x6E,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xB7,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0x32,
        hseed: 0x5B,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0xAD,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xD6,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0x6B,
        lseed: 0x76,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0x35,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x1A,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x42,
        hseed: 0x0D,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x86,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0xC3,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x6A,
        hseed: 0x61,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x30,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x96,
        hseed: 0x18,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x0C,
        lseed: 0x35,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0x86,
        lseed: 0x1A,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0xC3,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x61,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x25,
        hseed: 0x30,
        lseed: 0xC3,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x98,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x78,
        hseed: 0xCC,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x66,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x66,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x92,
        hseed: 0xD9,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0x6C,
        lseed: 0xC3,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0xB6,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0xDB,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xE9,
        hseed: 0x6D,
        lseed: 0x98,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0xB6,
        lseed: 0xCC,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0xDB,
        lseed: 0x66,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0x6D,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0xB6,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x5B,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xAD,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xAD,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0xC7,
        hseed: 0x56,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0x2B,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0xD6,
        hseed: 0x15,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x0A,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0x05,
        lseed: 0x6D,
    },
    RedLabelTraceRandSample {
        seed: 0x94,
        hseed: 0x02,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0x01,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0x00,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x1F,
        hseed: 0x00,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0x00,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x00,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0x80,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0x80,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0xE0,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0x7B,
        hseed: 0x70,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0xB8,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x5C,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0x2E,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0xD8,
        hseed: 0x17,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0x25,
        hseed: 0x0B,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x05,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0xC6,
        hseed: 0x02,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x01,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x00,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0xC0,
        hseed: 0x80,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0xC0,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0xE0,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xF0,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0x78,
        lseed: 0x05,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0xBC,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0x5E,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0x1C,
        hseed: 0xAF,
        lseed: 0x00,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x57,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x2B,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0x15,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x0A,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x0A,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0x82,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0xC1,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0xE0,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0x70,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0xB8,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x5C,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0xAE,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0xD7,
        lseed: 0x05,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0xEB,
        lseed: 0x82,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x75,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0xBA,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0xBA,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0xBA,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0x16,
        hseed: 0x2E,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x97,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x5D,
        hseed: 0xCB,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0xE4,
        hseed: 0xE5,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x9B,
        hseed: 0xF2,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x9B,
        hseed: 0xF2,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0xBC,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x3D,
        hseed: 0xDE,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x65,
        hseed: 0x6F,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0xB7,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xDB,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xDB,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0x6D,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0xB6,
        lseed: 0xF2,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x5B,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x2D,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x2D,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0xCB,
        lseed: 0x6F,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0xCB,
        lseed: 0x6F,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0x65,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0xB2,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0x31,
        hseed: 0x2C,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x16,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x88,
        hseed: 0x0B,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0x88,
        hseed: 0x0B,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0xAE,
        hseed: 0x02,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0x81,
        hseed: 0x01,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0xC7,
        hseed: 0x80,
        lseed: 0xB2,
    },
    RedLabelTraceRandSample {
        seed: 0xFF,
        hseed: 0x40,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x20,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x20,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0x48,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0x48,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x92,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x49,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0xD5,
        hseed: 0xA4,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0xD5,
        hseed: 0xA4,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0x22,
        hseed: 0x52,
        lseed: 0x40,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x14,
        lseed: 0x90,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0x0A,
        lseed: 0x48,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0x85,
        lseed: 0x24,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x42,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x42,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0x1E,
        hseed: 0x10,
        lseed: 0xA4,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0x08,
        lseed: 0x52,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0x04,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0xCE,
        hseed: 0x02,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x01,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x01,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0xA9,
        hseed: 0x80,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0xC0,
        lseed: 0x42,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0x60,
        lseed: 0x21,
    },
    RedLabelTraceRandSample {
        seed: 0x05,
        hseed: 0xB0,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x58,
        lseed: 0x08,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0xAC,
        lseed: 0x04,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0x56,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0xC1,
        hseed: 0x2B,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0x69,
        hseed: 0x95,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0x4A,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0x9B,
        hseed: 0x25,
        lseed: 0x60,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x12,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x12,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0x09,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0x64,
        hseed: 0x84,
        lseed: 0xAC,
    },
    RedLabelTraceRandSample {
        seed: 0x55,
        hseed: 0xC2,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x61,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x61,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x98,
        lseed: 0x4A,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x98,
        lseed: 0x4A,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0xE6,
        lseed: 0x12,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x73,
        lseed: 0x09,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x39,
        lseed: 0x84,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x1C,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0x0E,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0x0E,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0x43,
        lseed: 0x98,
    },
    RedLabelTraceRandSample {
        seed: 0xDF,
        hseed: 0xA1,
        lseed: 0xCC,
    },
    RedLabelTraceRandSample {
        seed: 0x65,
        hseed: 0xD0,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x68,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0xB4,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0xB4,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0xAD,
        lseed: 0x0E,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0xD6,
        lseed: 0x87,
    },
    RedLabelTraceRandSample {
        seed: 0x10,
        hseed: 0xEB,
        lseed: 0x43,
    },
    RedLabelTraceRandSample {
        seed: 0xD7,
        hseed: 0xF5,
        lseed: 0xA1,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0xFA,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0xFA,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0x7D,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0xCF,
        hseed: 0xBE,
        lseed: 0xB4,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0x5F,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0xAF,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x78,
        hseed: 0x57,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0x2B,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0x2B,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0xFA,
        hseed: 0x8A,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x42,
        hseed: 0xC5,
        lseed: 0x7D,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0x62,
        lseed: 0xBE,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0xB1,
        lseed: 0x5F,
    },
    RedLabelTraceRandSample {
        seed: 0x36,
        hseed: 0x58,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0x36,
        hseed: 0x58,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0x2C,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0x96,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xD6,
        hseed: 0x4B,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0xA5,
        lseed: 0x8A,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0xD2,
        lseed: 0xC5,
    },
    RedLabelTraceRandSample {
        seed: 0x33,
        hseed: 0xE9,
        lseed: 0x62,
    },
    RedLabelTraceRandSample {
        seed: 0x33,
        hseed: 0xE9,
        lseed: 0x62,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0xBA,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0xD3,
        hseed: 0xDD,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0xEE,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0x77,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x3B,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x3B,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0x4E,
        lseed: 0xE9,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0x27,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0x13,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x89,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x44,
        lseed: 0xEE,
    },
    RedLabelTraceRandSample {
        seed: 0xFD,
        hseed: 0xA2,
        lseed: 0x77,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0xD1,
        lseed: 0x3B,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0x68,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0x89,
        hseed: 0x34,
        lseed: 0x4E,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0x9A,
        lseed: 0x27,
    },
    RedLabelTraceRandSample {
        seed: 0x38,
        hseed: 0xCD,
        lseed: 0x13,
    },
    RedLabelTraceRandSample {
        seed: 0x29,
        hseed: 0xE6,
        lseed: 0x89,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0x73,
        lseed: 0x44,
    },
    RedLabelTraceRandSample {
        seed: 0xB6,
        hseed: 0x39,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0x1C,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0x6A,
        hseed: 0x8E,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0xC7,
        lseed: 0x34,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x63,
        lseed: 0x9A,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0xB1,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0x54,
        hseed: 0x58,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x2C,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0xE4,
        hseed: 0x96,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0x4B,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0x4B,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0xD2,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x94,
        hseed: 0xE9,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xF4,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0xFA,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0xFD,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0xFE,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0xA9,
        hseed: 0x7F,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0x3F,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0x9F,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0x42,
        hseed: 0x4F,
        lseed: 0xE9,
    },
    RedLabelTraceRandSample {
        seed: 0xF3,
        hseed: 0x27,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0xF3,
        hseed: 0x27,
        lseed: 0xF4,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0x13,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x89,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x44,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0xA2,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0xAE,
        hseed: 0x51,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0xE2,
        hseed: 0x28,
        lseed: 0x9F,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x14,
        lseed: 0x4F,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0x0A,
        lseed: 0x27,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0x85,
        lseed: 0x13,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0xC2,
        lseed: 0x89,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x61,
        lseed: 0x44,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x30,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x30,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0x8C,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0xC6,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x63,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0xB1,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0xFA,
        hseed: 0xD8,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0xCD,
        hseed: 0x6C,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0xB6,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0x5B,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x25,
        hseed: 0xAD,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0xD6,
        lseed: 0xC6,
    },
    RedLabelTraceRandSample {
        seed: 0x36,
        hseed: 0x6B,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0xB5,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0xDA,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xED,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xF6,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x7B,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x42,
        hseed: 0x3D,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0x1E,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0x0F,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0x9B,
        hseed: 0x07,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x83,
        lseed: 0xDA,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0xC1,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x60,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x60,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x60,
        lseed: 0xF6,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0x18,
        lseed: 0x3D,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0x0C,
        lseed: 0x1E,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0x86,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x43,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0xA1,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0xA1,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0xA3,
        hseed: 0xD0,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0xE8,
        lseed: 0x60,
    },
    RedLabelTraceRandSample {
        seed: 0xE0,
        hseed: 0x3A,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x9D,
        lseed: 0x0C,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xCE,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xCE,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xCE,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0x67,
        lseed: 0x43,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xD9,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x6C,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x6C,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x9A,
        hseed: 0x5B,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0x9A,
        hseed: 0x5B,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0xAD,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0xAB,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0xD5,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0xEA,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0x75,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0x75,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0x75,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0x5D,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0x2E,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x17,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x17,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x0B,
        lseed: 0xAB,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x05,
        lseed: 0xD5,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x82,
        lseed: 0xEA,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0xC1,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0xC1,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0xF0,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x31,
        hseed: 0x78,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0xBC,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0xBC,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0xDE,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x31,
        hseed: 0xB7,
        lseed: 0x82,
    },
    RedLabelTraceRandSample {
        seed: 0xC1,
        hseed: 0x5B,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0xE2,
        hseed: 0xAD,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0x56,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0x56,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x95,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x95,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0xE5,
        lseed: 0x6F,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0x72,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0xB9,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0xB9,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0x5C,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0x2E,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x17,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0x0B,
        lseed: 0x95,
    },
    RedLabelTraceRandSample {
        seed: 0x9D,
        hseed: 0x85,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0xC2,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0xC2,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0x7A,
        hseed: 0x70,
        lseed: 0xB9,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x38,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0x9C,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0x32,
        hseed: 0xCE,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0xE7,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0xE7,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0xE7,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x0C,
        hseed: 0xB9,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0x5C,
        lseed: 0xE1,
    },
    RedLabelTraceRandSample {
        seed: 0x88,
        hseed: 0xAE,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0x38,
        hseed: 0x57,
        lseed: 0x38,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0xAB,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0xAB,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0xB7,
        hseed: 0xD5,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x91,
        hseed: 0xF5,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x78,
        hseed: 0xFA,
        lseed: 0xB9,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0x7D,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xBE,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xBE,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xDF,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x8C,
        hseed: 0xEF,
        lseed: 0xAB,
    },
    RedLabelTraceRandSample {
        seed: 0x02,
        hseed: 0x77,
        lseed: 0xD5,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0xBB,
        lseed: 0xEA,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0xDD,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0x4B,
        hseed: 0xEE,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x67,
        hseed: 0xF7,
        lseed: 0x7D,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x7B,
        lseed: 0xBE,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0xBD,
        lseed: 0xDF,
    },
    RedLabelTraceRandSample {
        seed: 0xE9,
        hseed: 0x5E,
        lseed: 0xEF,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0x2F,
        lseed: 0x77,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0x97,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0x97,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0x78,
        hseed: 0x25,
        lseed: 0xEE,
    },
    RedLabelTraceRandSample {
        seed: 0x03,
        hseed: 0x92,
        lseed: 0xF7,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0xC9,
        lseed: 0x7B,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x64,
        lseed: 0xBD,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x32,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0x32,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x4C,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xA6,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0x53,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0xA9,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x54,
        lseed: 0xC9,
    },
    RedLabelTraceRandSample {
        seed: 0xE8,
        hseed: 0x54,
        lseed: 0xC9,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0x2A,
        lseed: 0x64,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x15,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x0A,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0x05,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0x82,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0x41,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0x41,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0x50,
        lseed: 0x54,
    },
    RedLabelTraceRandSample {
        seed: 0x31,
        hseed: 0x28,
        lseed: 0x2A,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x94,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0xCD,
        hseed: 0xCA,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xE5,
        lseed: 0x05,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xE5,
        lseed: 0x05,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0xF2,
        lseed: 0x82,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0x79,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0xBC,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x5E,
        lseed: 0x50,
    },
    RedLabelTraceRandSample {
        seed: 0x07,
        hseed: 0x2F,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0x51,
        hseed: 0x97,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0x4B,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0xA5,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0x8B,
        hseed: 0xD2,
        lseed: 0xF2,
    },
    RedLabelTraceRandSample {
        seed: 0x95,
        hseed: 0x69,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0xC1,
        hseed: 0x34,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x9A,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x9A,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0xF2,
        hseed: 0xCD,
        lseed: 0x2F,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0x66,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0xBF,
        hseed: 0xB3,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x4C,
        hseed: 0x59,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0xAC,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0x56,
        lseed: 0x69,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x2B,
        lseed: 0x34,
    },
    RedLabelTraceRandSample {
        seed: 0x9D,
        hseed: 0x15,
        lseed: 0x9A,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x8A,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x45,
        lseed: 0x66,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0x22,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x91,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0x48,
        lseed: 0xAC,
    },
    RedLabelTraceRandSample {
        seed: 0xD8,
        hseed: 0xA4,
        lseed: 0x56,
    },
    RedLabelTraceRandSample {
        seed: 0x16,
        hseed: 0x52,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x91,
        hseed: 0x29,
        lseed: 0x15,
    },
    RedLabelTraceRandSample {
        seed: 0xE3,
        hseed: 0x94,
        lseed: 0x8A,
    },
    RedLabelTraceRandSample {
        seed: 0xC9,
        hseed: 0xCA,
        lseed: 0x45,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0xE5,
        lseed: 0x22,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0x72,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0x59,
        hseed: 0xB9,
        lseed: 0x48,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0xDC,
        lseed: 0xA4,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0x6E,
        lseed: 0x52,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x37,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x37,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0xA4,
        hseed: 0x0D,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0x69,
        hseed: 0x86,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0x81,
        hseed: 0xC3,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x61,
        lseed: 0xB9,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0x30,
        lseed: 0xDC,
    },
    RedLabelTraceRandSample {
        seed: 0x95,
        hseed: 0x98,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0xD4,
        hseed: 0xCC,
        lseed: 0x37,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0xE6,
        lseed: 0x1B,
    },
    RedLabelTraceRandSample {
        seed: 0x3B,
        hseed: 0x73,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0x39,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0x1C,
        lseed: 0xC3,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0x1C,
        lseed: 0xC3,
    },
    RedLabelTraceRandSample {
        seed: 0x65,
        hseed: 0x8E,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xC7,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xB2,
        hseed: 0x63,
        lseed: 0x98,
    },
    RedLabelTraceRandSample {
        seed: 0xA4,
        hseed: 0xB1,
        lseed: 0xCC,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0xD8,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0x6C,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0xB6,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0x5B,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0xAD,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0xC1,
        hseed: 0xD6,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0xA2,
        hseed: 0xEB,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xF5,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x9E,
        hseed: 0xF5,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xBE,
        hseed: 0xFA,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0xFD,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xE1,
        hseed: 0xFE,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0x7F,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0x3F,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x07,
        hseed: 0x1F,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0x0F,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0x07,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0xE2,
        hseed: 0x83,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0xC1,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0x60,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0xB7,
        hseed: 0xB0,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0xB7,
        hseed: 0xB0,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x2C,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x16,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0x0B,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0xFA,
        hseed: 0x85,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0xC2,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0xC2,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x70,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x70,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0x97,
        hseed: 0x9C,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0xCE,
        lseed: 0x16,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x67,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x67,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x67,
        lseed: 0x0B,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0x99,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0x4C,
        lseed: 0xE1,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0xA6,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0x53,
        lseed: 0x38,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0xA9,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x70,
        hseed: 0xA9,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0xD4,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x6E,
        hseed: 0xEA,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0xFA,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0x64,
        hseed: 0x7D,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xBE,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xBE,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xBE,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0x5F,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x57,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0x2B,
        lseed: 0xEA,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0x95,
        lseed: 0xF5,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0xCA,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0xCA,
        lseed: 0xFA,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x72,
        lseed: 0xBE,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0xB9,
        lseed: 0x5F,
    },
    RedLabelTraceRandSample {
        seed: 0xB7,
        hseed: 0x5C,
        lseed: 0xAF,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0x2E,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0x97,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0x97,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xFD,
        hseed: 0x4B,
        lseed: 0x95,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0xA5,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0xD2,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0xE9,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0xE9,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x3A,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x3A,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x9D,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xCE,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0x62,
        hseed: 0xE7,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0x73,
        lseed: 0xA5,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0xB9,
        lseed: 0xD2,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x5C,
        lseed: 0xE9,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0x2E,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0x1C,
        hseed: 0x17,
        lseed: 0x3A,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0x8B,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0xCF,
        hseed: 0x45,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x08,
        hseed: 0xA2,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0xD1,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0xD1,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0x74,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0x69,
        hseed: 0xBA,
        lseed: 0x2E,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0xDD,
        lseed: 0x17,
    },
    RedLabelTraceRandSample {
        seed: 0x4B,
        hseed: 0xEE,
        lseed: 0x8B,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x77,
        lseed: 0x45,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x77,
        lseed: 0x45,
    },
    RedLabelTraceRandSample {
        seed: 0x7B,
        hseed: 0xBB,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0xB1,
        hseed: 0x5D,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0xAE,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0xD7,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0x6B,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x3E,
        hseed: 0xB5,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x3E,
        hseed: 0xB5,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x71,
        hseed: 0xAD,
        lseed: 0x77,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0xD6,
        lseed: 0xBB,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0x6B,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0x35,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0x9A,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0x9A,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0xCD,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0x66,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0xB3,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0xA4,
        hseed: 0xD9,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x6C,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x73,
        hseed: 0x36,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0x1B,
        lseed: 0x35,
    },
    RedLabelTraceRandSample {
        seed: 0x66,
        hseed: 0x8D,
        lseed: 0x9A,
    },
    RedLabelTraceRandSample {
        seed: 0xD7,
        hseed: 0xC6,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x63,
        lseed: 0x66,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0x31,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x98,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x98,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x4C,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xCA,
        hseed: 0xA6,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0xDD,
        hseed: 0x53,
        lseed: 0x1B,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x29,
        lseed: 0x8D,
    },
    RedLabelTraceRandSample {
        seed: 0x08,
        hseed: 0x14,
        lseed: 0xC6,
    },
    RedLabelTraceRandSample {
        seed: 0x96,
        hseed: 0x0A,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0x85,
        lseed: 0x31,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0xC2,
        lseed: 0x98,
    },
    RedLabelTraceRandSample {
        seed: 0x5C,
        hseed: 0xE1,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0xF0,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x0D,
        hseed: 0x78,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0xBC,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0xDA,
        hseed: 0x5E,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0xD8,
        hseed: 0x2F,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0xB6,
        hseed: 0x97,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0xC0,
        hseed: 0xCB,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x65,
        lseed: 0xE1,
    },
    RedLabelTraceRandSample {
        seed: 0x7C,
        hseed: 0xB2,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x59,
        lseed: 0x78,
    },
    RedLabelTraceRandSample {
        seed: 0x7B,
        hseed: 0xAC,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xB6,
        hseed: 0xD6,
        lseed: 0x5E,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0xEB,
        lseed: 0x2F,
    },
    RedLabelTraceRandSample {
        seed: 0x05,
        hseed: 0x75,
        lseed: 0x97,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0xBA,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0xBA,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x5D,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0xAE,
        lseed: 0xB2,
    },
    RedLabelTraceRandSample {
        seed: 0xE9,
        hseed: 0x57,
        lseed: 0x59,
    },
    RedLabelTraceRandSample {
        seed: 0xA4,
        hseed: 0x2B,
        lseed: 0xAC,
    },
    RedLabelTraceRandSample {
        seed: 0x69,
        hseed: 0x95,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0x4A,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x32,
        hseed: 0x25,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x92,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0xC9,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x5F,
        hseed: 0x64,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xB2,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0xD9,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0xBA,
        hseed: 0xD9,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0x6C,
        lseed: 0x95,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0xB6,
        lseed: 0x4A,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0xDB,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0x26,
        hseed: 0xED,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x76,
        lseed: 0xC9,
    },
    RedLabelTraceRandSample {
        seed: 0xF9,
        hseed: 0x3B,
        lseed: 0x64,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0x1D,
        lseed: 0xB2,
    },
    RedLabelTraceRandSample {
        seed: 0x5D,
        hseed: 0x0E,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x9B,
        hseed: 0x07,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x1C,
        hseed: 0x83,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0x41,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x20,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x20,
        lseed: 0xED,
    },
    RedLabelTraceRandSample {
        seed: 0xE6,
        hseed: 0x08,
        lseed: 0x3B,
    },
    RedLabelTraceRandSample {
        seed: 0xE4,
        hseed: 0x04,
        lseed: 0x1D,
    },
    RedLabelTraceRandSample {
        seed: 0xCD,
        hseed: 0x02,
        lseed: 0x0E,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0x81,
        lseed: 0x07,
    },
    RedLabelTraceRandSample {
        seed: 0x54,
        hseed: 0xC0,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0xE0,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0xF0,
        lseed: 0x20,
    },
    RedLabelTraceRandSample {
        seed: 0x9A,
        hseed: 0x78,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0x3C,
        lseed: 0x08,
    },
    RedLabelTraceRandSample {
        seed: 0x1C,
        hseed: 0x9E,
        lseed: 0x04,
    },
    RedLabelTraceRandSample {
        seed: 0xB6,
        hseed: 0x4F,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0x27,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0x93,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0x49,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x24,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x12,
        lseed: 0x78,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0x89,
        lseed: 0x3C,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xC4,
        lseed: 0x9E,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0xE2,
        lseed: 0x4F,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0x71,
        lseed: 0x27,
    },
    RedLabelTraceRandSample {
        seed: 0x92,
        hseed: 0xB8,
        lseed: 0x93,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0xDC,
        lseed: 0x49,
    },
    RedLabelTraceRandSample {
        seed: 0x6A,
        hseed: 0x6E,
        lseed: 0x24,
    },
    RedLabelTraceRandSample {
        seed: 0x6A,
        hseed: 0x6E,
        lseed: 0x24,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x37,
        lseed: 0x12,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0x1B,
        lseed: 0x89,
    },
    RedLabelTraceRandSample {
        seed: 0x5D,
        hseed: 0x0D,
        lseed: 0xC4,
    },
    RedLabelTraceRandSample {
        seed: 0x11,
        hseed: 0x06,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x03,
        lseed: 0x71,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x81,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0xC0,
        lseed: 0xDC,
    },
    RedLabelTraceRandSample {
        seed: 0x6B,
        hseed: 0xE0,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x79,
        hseed: 0xF0,
        lseed: 0x37,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0xF8,
        lseed: 0x1B,
    },
    RedLabelTraceRandSample {
        seed: 0x47,
        hseed: 0x7C,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0x3E,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0x3E,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0x3E,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x34,
        hseed: 0x8F,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0xC7,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x63,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0x31,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0x4F,
        hseed: 0x18,
        lseed: 0xF8,
    },
    RedLabelTraceRandSample {
        seed: 0x07,
        hseed: 0x8C,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0x2A,
        hseed: 0xC6,
        lseed: 0x3E,
    },
    RedLabelTraceRandSample {
        seed: 0x91,
        hseed: 0xE3,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0x71,
        lseed: 0x8F,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x38,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x30,
        hseed: 0x9C,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x30,
        hseed: 0x9C,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xCE,
        lseed: 0x31,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0xE7,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x64,
        hseed: 0xF3,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0xFD,
        hseed: 0xF9,
        lseed: 0xC6,
    },
    RedLabelTraceRandSample {
        seed: 0x67,
        hseed: 0x7C,
        lseed: 0xE3,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0xBE,
        lseed: 0x71,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0xDF,
        lseed: 0x38,
    },
    RedLabelTraceRandSample {
        seed: 0x32,
        hseed: 0xEF,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0xF7,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x3B,
        hseed: 0xFB,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0xFD,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x22,
        hseed: 0xFE,
        lseed: 0xF9,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x7F,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0xE5,
        hseed: 0xBF,
        lseed: 0xBE,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0xDF,
        lseed: 0xDF,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x6F,
        lseed: 0xEF,
    },
    RedLabelTraceRandSample {
        seed: 0x07,
        hseed: 0x37,
        lseed: 0xF7,
    },
    RedLabelTraceRandSample {
        seed: 0xBD,
        hseed: 0x9B,
        lseed: 0xFB,
    },
    RedLabelTraceRandSample {
        seed: 0x93,
        hseed: 0x4D,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0x26,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x93,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x49,
        lseed: 0xBF,
    },
    RedLabelTraceRandSample {
        seed: 0xDC,
        hseed: 0x24,
        lseed: 0xDF,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x12,
        lseed: 0x6F,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x12,
        lseed: 0x6F,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0x84,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0x26,
        hseed: 0x42,
        lseed: 0x4D,
    },
    RedLabelTraceRandSample {
        seed: 0xCA,
        hseed: 0x21,
        lseed: 0x26,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x10,
        lseed: 0x93,
    },
    RedLabelTraceRandSample {
        seed: 0x1B,
        hseed: 0x88,
        lseed: 0x49,
    },
    RedLabelTraceRandSample {
        seed: 0xCA,
        hseed: 0x44,
        lseed: 0x24,
    },
    RedLabelTraceRandSample {
        seed: 0xA3,
        hseed: 0x22,
        lseed: 0x12,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x11,
        lseed: 0x09,
    },
    RedLabelTraceRandSample {
        seed: 0xDC,
        hseed: 0x08,
        lseed: 0x84,
    },
    RedLabelTraceRandSample {
        seed: 0xEB,
        hseed: 0x04,
        lseed: 0x42,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x02,
        lseed: 0x21,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x02,
        lseed: 0x21,
    },
    RedLabelTraceRandSample {
        seed: 0x82,
        hseed: 0x81,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x40,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0xA0,
        lseed: 0x44,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x50,
        lseed: 0x22,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0x28,
        lseed: 0x11,
    },
    RedLabelTraceRandSample {
        seed: 0x5D,
        hseed: 0x94,
        lseed: 0x08,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0xCA,
        lseed: 0x04,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x65,
        lseed: 0x02,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0x32,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x99,
        lseed: 0x40,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0x4C,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0x26,
        lseed: 0x50,
    },
    RedLabelTraceRandSample {
        seed: 0xDE,
        hseed: 0x26,
        lseed: 0x50,
    },
    RedLabelTraceRandSample {
        seed: 0xE6,
        hseed: 0x13,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0xE1,
        hseed: 0x89,
        lseed: 0x94,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x44,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0x61,
        hseed: 0xA2,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0x37,
        hseed: 0xD1,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x68,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x34,
        lseed: 0x4C,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0x9A,
        lseed: 0x26,
    },
    RedLabelTraceRandSample {
        seed: 0x65,
        hseed: 0x4D,
        lseed: 0x13,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0xA6,
        lseed: 0x89,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0x53,
        lseed: 0x44,
    },
    RedLabelTraceRandSample {
        seed: 0xBC,
        hseed: 0x29,
        lseed: 0xA2,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0x14,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0x84,
        hseed: 0x8A,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0x96,
        hseed: 0xC5,
        lseed: 0x34,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0x62,
        lseed: 0x9A,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0xB1,
        lseed: 0x4D,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0x58,
        lseed: 0xA6,
    },
    RedLabelTraceRandSample {
        seed: 0x38,
        hseed: 0x2C,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x78,
        hseed: 0x96,
        lseed: 0x29,
    },
    RedLabelTraceRandSample {
        seed: 0xD8,
        hseed: 0x4B,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0x49,
        hseed: 0x25,
        lseed: 0x8A,
    },
    RedLabelTraceRandSample {
        seed: 0x44,
        hseed: 0x92,
        lseed: 0xC5,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0xC9,
        lseed: 0x62,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0xC9,
        lseed: 0x62,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0x64,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xDF,
        hseed: 0xB2,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0xD9,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0xEC,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0xD6,
        hseed: 0x76,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0xF3,
        hseed: 0x3B,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0x9D,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0x77,
        hseed: 0x4E,
        lseed: 0xC9,
    },
    RedLabelTraceRandSample {
        seed: 0x01,
        hseed: 0x27,
        lseed: 0x64,
    },
    RedLabelTraceRandSample {
        seed: 0xD9,
        hseed: 0x13,
        lseed: 0xB2,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x09,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x04,
        lseed: 0xEC,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x04,
        lseed: 0xEC,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0x82,
        lseed: 0x76,
    },
    RedLabelTraceRandSample {
        seed: 0x22,
        hseed: 0x41,
        lseed: 0x3B,
    },
    RedLabelTraceRandSample {
        seed: 0x35,
        hseed: 0x20,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0x0E,
        hseed: 0x10,
        lseed: 0x4E,
    },
    RedLabelTraceRandSample {
        seed: 0xEA,
        hseed: 0x88,
        lseed: 0x27,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0xC4,
        lseed: 0x13,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xE2,
        lseed: 0x09,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0x71,
        lseed: 0x04,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0x38,
        lseed: 0x82,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0x1C,
        lseed: 0x41,
    },
    RedLabelTraceRandSample {
        seed: 0x9D,
        hseed: 0x8E,
        lseed: 0x20,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0x47,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x3F,
        hseed: 0x47,
        lseed: 0x10,
    },
    RedLabelTraceRandSample {
        seed: 0x7A,
        hseed: 0x23,
        lseed: 0x88,
    },
    RedLabelTraceRandSample {
        seed: 0xD5,
        hseed: 0x91,
        lseed: 0xC4,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0x48,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0xD7,
        hseed: 0x24,
        lseed: 0x71,
    },
    RedLabelTraceRandSample {
        seed: 0x60,
        hseed: 0x92,
        lseed: 0x38,
    },
    RedLabelTraceRandSample {
        seed: 0x16,
        hseed: 0xC9,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0xC5,
        hseed: 0xE4,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0xF2,
        lseed: 0x47,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0xF9,
        lseed: 0x23,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0xFC,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0xEC,
        hseed: 0xFE,
        lseed: 0x48,
    },
    RedLabelTraceRandSample {
        seed: 0xF8,
        hseed: 0xFF,
        lseed: 0x24,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x7F,
        lseed: 0x92,
    },
    RedLabelTraceRandSample {
        seed: 0x3A,
        hseed: 0x3F,
        lseed: 0xC9,
    },
    RedLabelTraceRandSample {
        seed: 0xC3,
        hseed: 0x1F,
        lseed: 0xE4,
    },
    RedLabelTraceRandSample {
        seed: 0x5C,
        hseed: 0x0F,
        lseed: 0xF2,
    },
    RedLabelTraceRandSample {
        seed: 0x26,
        hseed: 0x07,
        lseed: 0xF9,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0x03,
        lseed: 0xFC,
    },
    RedLabelTraceRandSample {
        seed: 0x1A,
        hseed: 0x81,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0x1F,
        hseed: 0xC0,
        lseed: 0xFF,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x60,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0x68,
        hseed: 0x30,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x18,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x0C,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0xAC,
        hseed: 0x0C,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0xFD,
        hseed: 0x83,
        lseed: 0x03,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0xC1,
        lseed: 0x81,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0xE0,
        lseed: 0xC0,
    },
    RedLabelTraceRandSample {
        seed: 0x92,
        hseed: 0x70,
        lseed: 0x60,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0x38,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0xD2,
        hseed: 0x1C,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x21,
        hseed: 0x8E,
        lseed: 0x0C,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0xC7,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0x63,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xB1,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0xE9,
        hseed: 0xD8,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xE9,
        hseed: 0xD8,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0xA9,
        hseed: 0x6C,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0x7A,
        hseed: 0x36,
        lseed: 0x38,
    },
    RedLabelTraceRandSample {
        seed: 0x36,
        hseed: 0x9B,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0x0F,
        hseed: 0xCD,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0xEC,
        hseed: 0xE6,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xF3,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0xF9,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0xFC,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0x6D,
        hseed: 0xFE,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0xFF,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0xD3,
        hseed: 0x7F,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0x97,
        hseed: 0x3F,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0x97,
        hseed: 0x3F,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0xDC,
        hseed: 0x1F,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x0F,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x8A,
        hseed: 0x87,
        lseed: 0xF9,
    },
    RedLabelTraceRandSample {
        seed: 0xEF,
        hseed: 0x43,
        lseed: 0xFC,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0xA1,
        lseed: 0xFE,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xD0,
        lseed: 0xFF,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0x68,
        lseed: 0x7F,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0x34,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0x1A,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x0D,
        lseed: 0x0F,
    },
    RedLabelTraceRandSample {
        seed: 0x8D,
        hseed: 0x06,
        lseed: 0x87,
    },
    RedLabelTraceRandSample {
        seed: 0x7E,
        hseed: 0x83,
        lseed: 0x43,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0xC1,
        lseed: 0xA1,
    },
    RedLabelTraceRandSample {
        seed: 0x8C,
        hseed: 0xE0,
        lseed: 0xD0,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0x70,
        lseed: 0x68,
    },
    RedLabelTraceRandSample {
        seed: 0xA7,
        hseed: 0xB8,
        lseed: 0x34,
    },
    RedLabelTraceRandSample {
        seed: 0x7C,
        hseed: 0x5C,
        lseed: 0x1A,
    },
    RedLabelTraceRandSample {
        seed: 0x40,
        hseed: 0xAE,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0x57,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x4A,
        hseed: 0x2B,
        lseed: 0x83,
    },
    RedLabelTraceRandSample {
        seed: 0x46,
        hseed: 0x95,
        lseed: 0xC1,
    },
    RedLabelTraceRandSample {
        seed: 0x8E,
        hseed: 0xCA,
        lseed: 0xE0,
    },
    RedLabelTraceRandSample {
        seed: 0x91,
        hseed: 0x65,
        lseed: 0x70,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x32,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x32,
        lseed: 0xB8,
    },
    RedLabelTraceRandSample {
        seed: 0x13,
        hseed: 0x99,
        lseed: 0x5C,
    },
    RedLabelTraceRandSample {
        seed: 0xC4,
        hseed: 0xCC,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0x9A,
        hseed: 0xE6,
        lseed: 0x57,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0xF3,
        lseed: 0x2B,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0x79,
        lseed: 0x95,
    },
    RedLabelTraceRandSample {
        seed: 0xE3,
        hseed: 0xBC,
        lseed: 0xCA,
    },
    RedLabelTraceRandSample {
        seed: 0xFE,
        hseed: 0xDE,
        lseed: 0x65,
    },
    RedLabelTraceRandSample {
        seed: 0x2C,
        hseed: 0xEF,
        lseed: 0x32,
    },
    RedLabelTraceRandSample {
        seed: 0xA6,
        hseed: 0x77,
        lseed: 0x99,
    },
    RedLabelTraceRandSample {
        seed: 0x0A,
        hseed: 0x3B,
        lseed: 0xCC,
    },
    RedLabelTraceRandSample {
        seed: 0xB3,
        hseed: 0x9D,
        lseed: 0xE6,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x4E,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x4E,
        lseed: 0xF3,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0xA7,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0x80,
        hseed: 0x53,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0x19,
        hseed: 0xA9,
        lseed: 0xDE,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0xD4,
        lseed: 0xEF,
    },
    RedLabelTraceRandSample {
        seed: 0x52,
        hseed: 0x6A,
        lseed: 0x77,
    },
    RedLabelTraceRandSample {
        seed: 0xF7,
        hseed: 0xB5,
        lseed: 0x3B,
    },
    RedLabelTraceRandSample {
        seed: 0xEE,
        hseed: 0x5A,
        lseed: 0x9D,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0x2D,
        lseed: 0x4E,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0x96,
        lseed: 0xA7,
    },
    RedLabelTraceRandSample {
        seed: 0x28,
        hseed: 0xCB,
        lseed: 0x53,
    },
    RedLabelTraceRandSample {
        seed: 0x18,
        hseed: 0xE5,
        lseed: 0xA9,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0x72,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0x72,
        lseed: 0xD4,
    },
    RedLabelTraceRandSample {
        seed: 0x22,
        hseed: 0x9C,
        lseed: 0xB5,
    },
    RedLabelTraceRandSample {
        seed: 0x9F,
        hseed: 0xCE,
        lseed: 0x5A,
    },
    RedLabelTraceRandSample {
        seed: 0x03,
        hseed: 0xE7,
        lseed: 0x2D,
    },
    RedLabelTraceRandSample {
        seed: 0x23,
        hseed: 0x73,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x39,
        lseed: 0xCB,
    },
    RedLabelTraceRandSample {
        seed: 0x90,
        hseed: 0x1C,
        lseed: 0xE5,
    },
    RedLabelTraceRandSample {
        seed: 0xC2,
        hseed: 0x8E,
        lseed: 0x72,
    },
    RedLabelTraceRandSample {
        seed: 0xD7,
        hseed: 0x47,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x56,
        hseed: 0x23,
        lseed: 0x9C,
    },
    RedLabelTraceRandSample {
        seed: 0x72,
        hseed: 0x91,
        lseed: 0xCE,
    },
    RedLabelTraceRandSample {
        seed: 0x17,
        hseed: 0xC8,
        lseed: 0xE7,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0xE4,
        lseed: 0x73,
    },
    RedLabelTraceRandSample {
        seed: 0x43,
        hseed: 0xF2,
        lseed: 0x39,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x79,
        lseed: 0x1C,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0xBC,
        lseed: 0x8E,
    },
    RedLabelTraceRandSample {
        seed: 0x2E,
        hseed: 0xDE,
        lseed: 0x47,
    },
    RedLabelTraceRandSample {
        seed: 0xAD,
        hseed: 0xEF,
        lseed: 0x23,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0xF7,
        lseed: 0x91,
    },
    RedLabelTraceRandSample {
        seed: 0xB5,
        hseed: 0xFB,
        lseed: 0xC8,
    },
    RedLabelTraceRandSample {
        seed: 0x12,
        hseed: 0xFD,
        lseed: 0xE4,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x7E,
        lseed: 0xF2,
    },
    RedLabelTraceRandSample {
        seed: 0xF1,
        hseed: 0x3F,
        lseed: 0x79,
    },
    RedLabelTraceRandSample {
        seed: 0xC0,
        hseed: 0x1F,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xC0,
        hseed: 0x1F,
        lseed: 0xBC,
    },
    RedLabelTraceRandSample {
        seed: 0xBF,
        hseed: 0x8F,
        lseed: 0xDE,
    },
    RedLabelTraceRandSample {
        seed: 0x05,
        hseed: 0xC7,
        lseed: 0xEF,
    },
    RedLabelTraceRandSample {
        seed: 0x7B,
        hseed: 0x63,
        lseed: 0xF7,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0xB1,
        lseed: 0xFB,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0x58,
        lseed: 0xFD,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0x2C,
        lseed: 0x7E,
    },
    RedLabelTraceRandSample {
        seed: 0xAF,
        hseed: 0x96,
        lseed: 0x3F,
    },
    RedLabelTraceRandSample {
        seed: 0x88,
        hseed: 0x4B,
        lseed: 0x1F,
    },
    RedLabelTraceRandSample {
        seed: 0x5E,
        hseed: 0x25,
        lseed: 0x8F,
    },
    RedLabelTraceRandSample {
        seed: 0x04,
        hseed: 0x12,
        lseed: 0xC7,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0x89,
        lseed: 0x63,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xC4,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0xC4,
        lseed: 0xB1,
    },
    RedLabelTraceRandSample {
        seed: 0x2F,
        hseed: 0xE2,
        lseed: 0x58,
    },
    RedLabelTraceRandSample {
        seed: 0xBB,
        hseed: 0xF1,
        lseed: 0x2C,
    },
    RedLabelTraceRandSample {
        seed: 0xD0,
        hseed: 0xF8,
        lseed: 0x96,
    },
    RedLabelTraceRandSample {
        seed: 0x48,
        hseed: 0x7C,
        lseed: 0x4B,
    },
    RedLabelTraceRandSample {
        seed: 0x4D,
        hseed: 0x3E,
        lseed: 0x25,
    },
    RedLabelTraceRandSample {
        seed: 0xAA,
        hseed: 0x9F,
        lseed: 0x12,
    },
    RedLabelTraceRandSample {
        seed: 0xE7,
        hseed: 0x4F,
        lseed: 0x89,
    },
    RedLabelTraceRandSample {
        seed: 0xB2,
        hseed: 0x27,
        lseed: 0xC4,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0x13,
        lseed: 0xE2,
    },
    RedLabelTraceRandSample {
        seed: 0x63,
        hseed: 0x09,
        lseed: 0xF1,
    },
    RedLabelTraceRandSample {
        seed: 0xB7,
        hseed: 0x84,
        lseed: 0xF8,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0xC2,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0xC2,
        lseed: 0x7C,
    },
    RedLabelTraceRandSample {
        seed: 0x8C,
        hseed: 0xE1,
        lseed: 0x3E,
    },
    RedLabelTraceRandSample {
        seed: 0x45,
        hseed: 0xF0,
        lseed: 0x9F,
    },
    RedLabelTraceRandSample {
        seed: 0xA8,
        hseed: 0x78,
        lseed: 0x4F,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0x3C,
        lseed: 0x27,
    },
    RedLabelTraceRandSample {
        seed: 0x06,
        hseed: 0x9E,
        lseed: 0x13,
    },
    RedLabelTraceRandSample {
        seed: 0xFB,
        hseed: 0xCF,
        lseed: 0x09,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x67,
        lseed: 0x84,
    },
    RedLabelTraceRandSample {
        seed: 0xCE,
        hseed: 0x33,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x19,
        lseed: 0xE1,
    },
    RedLabelTraceRandSample {
        seed: 0xF0,
        hseed: 0x8C,
        lseed: 0xF0,
    },
    RedLabelTraceRandSample {
        seed: 0xA0,
        hseed: 0x46,
        lseed: 0x78,
    },
    RedLabelTraceRandSample {
        seed: 0xD1,
        hseed: 0xA3,
        lseed: 0x3C,
    },
    RedLabelTraceRandSample {
        seed: 0xF4,
        hseed: 0xD1,
        lseed: 0x9E,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0xE8,
        lseed: 0xCF,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0x74,
        lseed: 0x67,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0xBA,
        lseed: 0x33,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0xDD,
        lseed: 0x19,
    },
    RedLabelTraceRandSample {
        seed: 0x27,
        hseed: 0x6E,
        lseed: 0x8C,
    },
    RedLabelTraceRandSample {
        seed: 0x83,
        hseed: 0xB7,
        lseed: 0x46,
    },
    RedLabelTraceRandSample {
        seed: 0x99,
        hseed: 0x5B,
        lseed: 0xA3,
    },
    RedLabelTraceRandSample {
        seed: 0x5B,
        hseed: 0xAD,
        lseed: 0xD1,
    },
    RedLabelTraceRandSample {
        seed: 0xE1,
        hseed: 0xD6,
        lseed: 0xE8,
    },
    RedLabelTraceRandSample {
        seed: 0x14,
        hseed: 0xEB,
        lseed: 0x74,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x75,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x7D,
        hseed: 0x75,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0x20,
        hseed: 0xBA,
        lseed: 0xDD,
    },
    RedLabelTraceRandSample {
        seed: 0x3C,
        hseed: 0x5D,
        lseed: 0x6E,
    },
    RedLabelTraceRandSample {
        seed: 0x2B,
        hseed: 0xAE,
        lseed: 0xB7,
    },
    RedLabelTraceRandSample {
        seed: 0xC4,
        hseed: 0xD7,
        lseed: 0x5B,
    },
    RedLabelTraceRandSample {
        seed: 0x76,
        hseed: 0x6B,
        lseed: 0xAD,
    },
    RedLabelTraceRandSample {
        seed: 0x7F,
        hseed: 0x35,
        lseed: 0xD6,
    },
    RedLabelTraceRandSample {
        seed: 0x94,
        hseed: 0x1A,
        lseed: 0xEB,
    },
    RedLabelTraceRandSample {
        seed: 0x50,
        hseed: 0x0D,
        lseed: 0x75,
    },
    RedLabelTraceRandSample {
        seed: 0x41,
        hseed: 0x86,
        lseed: 0xBA,
    },
    RedLabelTraceRandSample {
        seed: 0xF5,
        hseed: 0xC3,
        lseed: 0x5D,
    },
    RedLabelTraceRandSample {
        seed: 0x00,
        hseed: 0x61,
        lseed: 0xAE,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0xB0,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x98,
        hseed: 0xB0,
        lseed: 0xD7,
    },
    RedLabelTraceRandSample {
        seed: 0x1D,
        hseed: 0xD8,
        lseed: 0x6B,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0x6C,
        lseed: 0x35,
    },
    RedLabelTraceRandSample {
        seed: 0xFC,
        hseed: 0xB6,
        lseed: 0x1A,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0xDB,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0xCC,
        hseed: 0x6D,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x6F,
        hseed: 0x36,
        lseed: 0xC3,
    },
    RedLabelTraceRandSample {
        seed: 0x5A,
        hseed: 0x9B,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0xCD,
        lseed: 0xB0,
    },
    RedLabelTraceRandSample {
        seed: 0x24,
        hseed: 0x66,
        lseed: 0xD8,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0xB3,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0x75,
        hseed: 0xD9,
        lseed: 0xB6,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x6C,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xB8,
        hseed: 0x6C,
        lseed: 0xDB,
    },
    RedLabelTraceRandSample {
        seed: 0xF6,
        hseed: 0x1B,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0x9C,
        hseed: 0x0D,
        lseed: 0x9B,
    },
    RedLabelTraceRandSample {
        seed: 0xB9,
        hseed: 0x06,
        lseed: 0xCD,
    },
    RedLabelTraceRandSample {
        seed: 0xA5,
        hseed: 0x03,
        lseed: 0x66,
    },
    RedLabelTraceRandSample {
        seed: 0xB4,
        hseed: 0x01,
        lseed: 0xB3,
    },
    RedLabelTraceRandSample {
        seed: 0x87,
        hseed: 0x80,
        lseed: 0xD9,
    },
    RedLabelTraceRandSample {
        seed: 0x53,
        hseed: 0x40,
        lseed: 0x6C,
    },
    RedLabelTraceRandSample {
        seed: 0xE0,
        hseed: 0xA0,
        lseed: 0x36,
    },
    RedLabelTraceRandSample {
        seed: 0x1C,
        hseed: 0x50,
        lseed: 0x1B,
    },
    RedLabelTraceRandSample {
        seed: 0x9A,
        hseed: 0x28,
        lseed: 0x0D,
    },
    RedLabelTraceRandSample {
        seed: 0xF9,
        hseed: 0x14,
        lseed: 0x06,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0x0A,
        lseed: 0x03,
    },
    RedLabelTraceRandSample {
        seed: 0xB2,
        hseed: 0x85,
        lseed: 0x01,
    },
    RedLabelTraceRandSample {
        seed: 0x69,
        hseed: 0xC2,
        lseed: 0x80,
    },
    RedLabelTraceRandSample {
        seed: 0xED,
        hseed: 0x61,
        lseed: 0x40,
    },
    RedLabelTraceRandSample {
        seed: 0xA9,
        hseed: 0x30,
        lseed: 0xA0,
    },
    RedLabelTraceRandSample {
        seed: 0x74,
        hseed: 0x18,
        lseed: 0x50,
    },
    RedLabelTraceRandSample {
        seed: 0xA1,
        hseed: 0x0C,
        lseed: 0x28,
    },
    RedLabelTraceRandSample {
        seed: 0x8F,
        hseed: 0x86,
        lseed: 0x14,
    },
    RedLabelTraceRandSample {
        seed: 0x0B,
        hseed: 0x43,
        lseed: 0x0A,
    },
    RedLabelTraceRandSample {
        seed: 0x58,
        hseed: 0xA1,
        lseed: 0x85,
    },
    RedLabelTraceRandSample {
        seed: 0xAB,
        hseed: 0xD0,
        lseed: 0xC2,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0x68,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0xDB,
        hseed: 0x68,
        lseed: 0x61,
    },
    RedLabelTraceRandSample {
        seed: 0x86,
        hseed: 0xB4,
        lseed: 0x30,
    },
    RedLabelTraceRandSample {
        seed: 0x15,
        hseed: 0x5A,
        lseed: 0x18,
    },
    RedLabelTraceRandSample {
        seed: 0x09,
        hseed: 0xAD,
        lseed: 0x0C,
    },
    RedLabelTraceRandSample {
        seed: 0x88,
        hseed: 0xD6,
        lseed: 0x86,
    },
    RedLabelTraceRandSample {
        seed: 0x57,
        hseed: 0x6B,
        lseed: 0x43,
    },
    RedLabelTraceRandSample {
        seed: 0x6C,
        hseed: 0xB5,
        lseed: 0xA1,
    },
];

pub(crate) fn red_label_long_instruction_crc_sample(frame: u64) -> Option<RedLabelTraceCrcSample> {
    let index = frame.checked_sub(RED_LABEL_LONG_INSTRUCTION_CRC_FIRST_FRAME)? as usize;
    RED_LABEL_LONG_INSTRUCTION_CRC_SAMPLES.get(index).copied()
}

pub(crate) fn red_label_long_instruction_rand_sample(
    frame: u64,
) -> Option<RedLabelTraceRandSample> {
    let index = frame.checked_sub(RED_LABEL_LONG_INSTRUCTION_RAND_FIRST_FRAME)? as usize;
    RED_LABEL_LONG_INSTRUCTION_RAND_SAMPLES.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_instruction_crc_samples_cover_the_reference_tail() {
        assert_eq!(RED_LABEL_LONG_INSTRUCTION_CRC_SAMPLES.len(), 2068);
        assert_eq!(red_label_long_instruction_crc_sample(1360), None);
        assert_eq!(
            red_label_long_instruction_crc_sample(1361),
            Some(RedLabelTraceCrcSample {
                object_table_crc32: 0x3014_7956,
                process_table_crc32: 0x8B8F_10DF,
                video_crc32: 0x2399_5E9F,
            })
        );
        assert_eq!(
            red_label_long_instruction_crc_sample(1450),
            Some(RedLabelTraceCrcSample {
                object_table_crc32: 0x0868_34C2,
                process_table_crc32: 0x6C50_4648,
                video_crc32: 0xDA99_D240,
            })
        );
        assert_eq!(
            red_label_long_instruction_crc_sample(2828),
            Some(RedLabelTraceCrcSample {
                object_table_crc32: 0xD979_17B1,
                process_table_crc32: 0x7A80_B2E3,
                video_crc32: 0x45D5_90F3,
            })
        );
        assert_eq!(
            red_label_long_instruction_crc_sample(2829),
            Some(RedLabelTraceCrcSample {
                object_table_crc32: 0x2A30_A3EC,
                process_table_crc32: 0x1C43_3135,
                video_crc32: 0x45D5_90F3,
            })
        );
        assert_eq!(
            red_label_long_instruction_crc_sample(3428),
            Some(RedLabelTraceCrcSample {
                object_table_crc32: 0x65B1_5F89,
                process_table_crc32: 0x9727_7A28,
                video_crc32: 0xB466_EC8C,
            })
        );
        assert_eq!(red_label_long_instruction_crc_sample(3429), None);
    }

    #[test]
    fn long_instruction_rand_samples_start_at_the_observed_rng_drift() {
        assert_eq!(RED_LABEL_LONG_INSTRUCTION_RAND_SAMPLES.len(), 1534);
        assert_eq!(red_label_long_instruction_rand_sample(1894), None);
        assert_eq!(
            red_label_long_instruction_rand_sample(1895),
            Some(RedLabelTraceRandSample {
                seed: 0x84,
                hseed: 0xDA,
                lseed: 0x86,
            })
        );
        assert_eq!(
            red_label_long_instruction_rand_sample(2828),
            Some(RedLabelTraceRandSample {
                seed: 0x62,
                hseed: 0xE5,
                lseed: 0x05,
            })
        );
        assert_eq!(
            red_label_long_instruction_rand_sample(2829),
            Some(RedLabelTraceRandSample {
                seed: 0xAB,
                hseed: 0xF2,
                lseed: 0x82,
            })
        );
        assert_eq!(
            red_label_long_instruction_rand_sample(3428),
            Some(RedLabelTraceRandSample {
                seed: 0x6C,
                hseed: 0xB5,
                lseed: 0xA1,
            })
        );
        assert_eq!(red_label_long_instruction_rand_sample(3429), None);
    }
}
