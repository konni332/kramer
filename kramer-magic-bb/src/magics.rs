pub const ROOK_MAGICS: [u64; 64] = [
    0x7080008020104008,
    0x0840031003a00140,
    0x2200088242012010,
    0x0880100004080080,
    0x0100050010020800,
    0x1480010400800200,
    0x1080120008804900,
    0x0200010404408036,
    0x0010800040002090,
    0x2000400040201004,
    0x3100802000100080,
    0x0300800800100080,
    0x8085000500580010,
    0x0082800c00120080,
    0x5001000100020004,
    0x8201002100084092,
    0x9040008010204082,
    0x8010004020004000,
    0x0020010040241100,
    0x0000090020100500,
    0x007c008080080004,
    0x0024008080040200,
    0x9200010100020004,
    0x000002000040a104,
    0x9400400880008020,
    0x0420100340044120,
    0x0000100080802000,
    0x0048000880801000,
    0x211c028080080004,
    0x8000040801201040,
    0x0020100402020008,
    0x0083802080004100,
    0x0040004824800084,
    0x2000a01002400240,
    0x1040100080802000,
    0x0024100025000900,
    0x4241001205000800,
    0x2444040080800200,
    0x4828823094002108,
    0x400024008a001841,
    0x0040804004228000,
    0x8024500020024004,
    0x0402001040820020,
    0x2008001000088080,
    0x002b000800110006,
    0x0011002400490002,
    0x0000880110040002,
    0x0a400f19804a0004,
    0x0800400080003080,
    0x0010004000200440,
    0x8240812000100480,
    0x000a004024081200,
    0x0c04080011000500,
    0x8041000400080300,
    0x1090a80230010400,
    0x4080bc2108804200,
    0xc000108001002041,
    0x0800204081001202,
    0x0211110340200009,
    0x0100205000290015,
    0x0495000402080011,
    0x2c81000208040001,
    0x00a32a081081100c,
    0x0500022401024882,
];

pub const BISHOP_MAGICS: [u64; 64] = [
    0x004002042c102041,
    0x0220841410803084,
    0x2008480040880400,
    0x0044050a10604042,
    0x0001104081803000,
    0x0021012051c00404,
    0x0008820802410900,
    0x4021041101082200,
    0x00824004080a0a60,
    0x0211a21024250044,
    0x001008082040809c,
    0x2000208903003404,
    0x0000420210080000,
    0x1002420802080010,
    0x08341088048a6000,
    0x0010020094018800,
    0x2006004108020400,
    0x0050000504181040,
    0x309006282044c008,
    0x20809408020040b0,
    0x2001001820080008,
    0xc202040040500402,
    0x18012c0084100201,
    0x0400500201088804,
    0x8004200240035400,
    0x0041442010040801,
    0x8008220030040041,
    0x9024010000200980,
    0x3801001109004004,
    0x030440806900a011,
    0xc084010200480200,
    0x0102008008240100,
    0x0048084081848402,
    0x0180821080881044,
    0x0000444800500320,
    0x00c0020080080080,
    0x0104040400101100,
    0x002000808003025a,
    0x00d4480184006404,
    0x1088a20200004105,
    0x00822120088020a4,
    0x1000a80108081040,
    0x0240201048005008,
    0x0000884208001080,
    0x3000010202010420,
    0x0814301042001240,
    0x102114040042c086,
    0x0010210040804100,
    0x0004040108081600,
    0x1010411808620000,
    0x0400c20042680074,
    0x4080012104880006,
    0x8400084010410002,
    0x0408c81090808080,
    0x8010108208084080,
    0x2020054103051404,
    0x0002020610820800,
    0x8811482104100460,
    0x204008002422080e,
    0x100800b200208800,
    0x0040020040104100,
    0x0000021020018100,
    0x4400100401184200,
    0x1040080081020020,
];

#[cfg(test)]
mod tests {
    use crate::{
        bishop_attacks::bishop_attacks_on_the_fly,
        bishop_mask::bishop_mask,
        magics::{BISHOP_MAGICS, ROOK_MAGICS},
        rook_attacks::rook_attacks_on_the_fly,
        rook_mask::rook_mask,
    };

    #[test]
    fn verify_rook_magics() {
        let rook_magics: [u64; 64] = ROOK_MAGICS;
        #[allow(clippy::needless_range_loop)]
        for sq in 0..64usize {
            let mask = rook_mask(sq as u8);
            let bits = mask.count_ones() as u8;
            let shift = 64 - bits;
            let table_size = 1usize << bits;

            // build the table for this square
            let mut table = vec![u64::MAX; table_size];

            let mut subset = 0u64;
            loop {
                let attacks = rook_attacks_on_the_fly(sq as u8, subset);
                let idx = (subset.wrapping_mul(rook_magics[sq]) >> shift) as usize;

                if table[idx] == u64::MAX {
                    table[idx] = attacks;
                } else if table[idx] != attacks {
                    panic!(
                        "ROOK collision at sq={} subset={:#018x} idx={} expected={:#018x} got={:#018x}",
                        sq, subset, idx, attacks, table[idx]
                    );
                }

                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0 {
                    break;
                }
            }
        }

        println!("ROOK_MAGICS: all 64 squares ok");
    }

    #[test]
    fn verify_bishop_magics() {
        let bishop_magics: [u64; 64] = BISHOP_MAGICS;

        #[allow(clippy::needless_range_loop)]
        for sq in 0..64usize {
            let mask = bishop_mask(sq as u8);
            let bits = mask.count_ones() as u8;
            let shift = 64 - bits;
            let table_size = 1usize << bits;

            let mut table = vec![u64::MAX; table_size];

            let mut subset = 0u64;
            loop {
                let attacks = bishop_attacks_on_the_fly(sq as u8, subset);
                let idx = (subset.wrapping_mul(bishop_magics[sq]) >> shift) as usize;

                if table[idx] == u64::MAX {
                    table[idx] = attacks;
                } else if table[idx] != attacks {
                    panic!(
                        "BISHOP collision at sq={} subset={:#018x} idx={} expected={:#018x} got={:#018x}",
                        sq, subset, idx, attacks, table[idx]
                    );
                }

                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0 {
                    break;
                }
            }
        }

        println!("BISHOP_MAGICS: all 64 squares ok");
    }
}
