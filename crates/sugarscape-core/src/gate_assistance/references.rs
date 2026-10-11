//! Approved compact witnesses and portable full Row-array comparisons.
//! The ignored numerical artifacts are separately rehashed at the collection freeze.
use super::{canonical::canonical_bytes, *};
use serde::de::DeserializeOwned;
use serde_json::Value;
// Copied only from independently approved portable-row-identities.json.
const ROW_IDENTITIES: &[(&str, u64, &str, &str)] = &[
    (
        "rows-00-k1-b0-d0.json",
        14267236,
        "dd5493ea4eb961a0ea996740d8bc02642e6cab04eb9499edb913492e3269da74",
        "a49ee36be20e71bc963d5ebba706fa41e4505c33dd9c4695b5149f06f2b4dbf4",
    ),
    (
        "rows-01-k1-b0-d1.json",
        14276774,
        "989a2874450cfacdf21082e492b8451a0c9c17ad28e3e8ae3bd11c7f8c0ab3f9",
        "5d697a6c477bc630c12f8b8bd8a639b307173ec909099da89d241d94d19e5335",
    ),
    (
        "rows-02-k1-b3-d0.json",
        14263979,
        "a633acfc3321027ee8a5c30611d3a045c8185cff1173a11adbf30ce9ca856ca6",
        "9ecbad3365a015da9ad0546c60aeda9e87e1453988c3e67b415683a251c64878",
    ),
    (
        "rows-03-k1-b3-d1.json",
        14276278,
        "2be2e9a5433e10a59f436257416b216d1be1e66e9dce9822fddf402598d130c0",
        "e8b9a8f1e885c2af36c1324603e05d0aa434c16c5222ef485306286c59d5f978",
    ),
    (
        "rows-04-k1-b6-d0.json",
        14261944,
        "fada3f0db089e2219e6530a4a801fee04a6dbf97f289bd33682621cfbfe4cf4e",
        "436f9a327d7cbcc0dfb2a91930b2f1fba4a2dbe89f77362abe17ad5df393a8cb",
    ),
    (
        "rows-05-k1-b6-d1.json",
        14275838,
        "79481fb9cba54b0aa51a3866034a9b5cc6a90effcad4fcf7edaf8a65f4c85822",
        "d51c15038a837d760abe5e650da6ad99cb85630055db46f0f5dd9dea895d4934",
    ),
    (
        "rows-06-k2-b0-d0.json",
        14286269,
        "cd681bdc2f79275b4fe3f9940e6edbd0ac99365eb003326497490aa496350953",
        "c08f438a610ca9dbcdff15466d51d6a401a790c48803c2a10042dadc2dea6717",
    ),
    (
        "rows-07-k2-b0-d1.json",
        14296821,
        "5f6fa7df9595f45e2d8548e2813398e6c7effd24ca6304ffe312896ab9c7d5d7",
        "e9af971b0ce2c81886386ef7617d1b36202ea987073f51573961a66d8d02e825",
    ),
    (
        "rows-08-k2-b3-d0.json",
        14283396,
        "189a4a4629547b21c9dafc3d15aedea0c871bfa32c31451f667776234d3ee2ac",
        "9f9672f18a3089ce7fc92a9d8965d3bbec489ff4cac68179a9654b6e43eaae59",
    ),
    (
        "rows-09-k2-b3-d1.json",
        14296305,
        "b90c3f10c593228544c90c8397dc69997eaa3ffd521e13efc79f2bd8e24ac557",
        "668422275d9b083f6291c47ef346137df9f54e5db39acc791c322d69e4ff56a7",
    ),
    (
        "rows-10-k2-b6-d0.json",
        14282503,
        "9d89d340c335b1305da6bfb9e9fb23c99c333ea823fedd1536cdfbcfb6104d95",
        "84b34e67fc61856fbba5ed9dd2149458b3fc8f5fe2da3a80f710869ed29a629f",
    ),
    (
        "rows-11-k2-b6-d1.json",
        14296209,
        "e370da9bddbbb19f4ad9b071450e1adcddcbdcbd6ee092d1dd75e0cbcdf88c1e",
        "95a7242a72332500aa5afd39bba645b61e08bb70969ae56162e00bbbb3e30fb3",
    ),
    (
        "rows-12-k4-b0-d0.json",
        14352527,
        "3fc747f6a738175ffd5cb0b112003a58c0c40a3c8aa3384a533dad4ea845b5f8",
        "6f165a9e82e0aee87a49ce29e674bb4d53436fbdc42d8b2eb314aeefc4940264",
    ),
    (
        "rows-13-k4-b0-d1.json",
        14361408,
        "bbe6979e0762e6117f22d6aca138590d46f91be46dc523addc7f55dfad43d62e",
        "32eff10810726becb3eb734b195a27468d4888ed1d130a03f52be252803ea4f6",
    ),
    (
        "rows-14-k4-b3-d0.json",
        14349204,
        "3aad30dbf86e37ec80afdedb803f5251a98b65f152ae7ba61f2eac677faf40fe",
        "7a349ca2ea3f9f9e4694c075640a81f2f2b1dedbf21eecd92b0dd3836ee7ea76",
    ),
    (
        "rows-15-k4-b3-d1.json",
        14360808,
        "91dfea111d3cd6a1ee4ab7d6b5ed5092399b8b1372de97e3a09d05f3f9681d94",
        "4ecf20d0037b53be5cbd28409993aee069e4f6815052dae0d39d1f62d290353c",
    ),
    (
        "rows-16-k4-b6-d0.json",
        14348312,
        "1438ae64c9a164e3878254df125c60343fa633e77c03c93479724ede55662fef",
        "837cfc2c7bf0b8658cbafe7286dd999d612773123938ad3a6c50b0508cdc0baf",
    ),
    (
        "rows-17-k4-b6-d1.json",
        14360600,
        "48b9d8e0dfc24a3f98d4027bd4dcfc799637cf5e5c5055b9854fd76fbf1a4ec7",
        "7662970e58ce1f09f0baf3eda2f7e3d3954f84efbcfce4ff25a462df0b5d40af",
    ),
];
pub(super) fn decode<T: DeserializeOwned>(value: &Value, context: &str) -> Result<T, Error> {
    serde_json::from_value(value.clone())
        .map_err(|e| Error::InvalidReport(format!("reference {context}: {e}")))
}
pub(super) struct References {
    pub kernel: Value,
    pub game: Value,
    pub limits: Value,
}
impl References {
    pub fn load() -> Result<Self, Error> {
        let mut fixtures = Vec::new();
        for (path, bytes) in REFERENCE_INPUTS {
            let value: Value = serde_json::from_slice(bytes)
                .map_err(|e| Error::InvalidReport(format!("{path}: {e}")))?;
            if value["finalization"]["status"] != "independently_approved"
                || value["protocol_version"] != 1
            {
                return Err(Error::InvalidReport(format!("unapproved reference {path}")));
            }
            let strata: Vec<Stratum> = decode(&value["ordered_strata"], path)?;
            if strata != registered_strata() {
                return Err(Error::InvalidReport(format!("reference strata {path}")));
            }
            fixtures.push(value);
        }
        let mut fixtures = fixtures.into_iter();
        let refs = Self {
            kernel: fixtures
                .next()
                .ok_or_else(|| Error::InvalidReport("kernel reference absent".into()))?,
            game: fixtures
                .next()
                .ok_or_else(|| Error::InvalidReport("game reference absent".into()))?,
            limits: fixtures
                .next()
                .ok_or_else(|| Error::InvalidReport("limits reference absent".into()))?,
        };
        for (name, array) in [
            ("rows", &refs.kernel["ordered_row_receipts"]),
            ("game", &refs.game["panels"]),
            ("limits", &refs.limits["panels"]),
        ] {
            let array = array
                .as_array()
                .ok_or_else(|| Error::InvalidReport(format!("reference {name} panels")))?;
            if array.len() != 18 {
                return Err(Error::InvalidReport(format!("reference {name} count")));
            }
            for (i, s) in registered_strata().iter().enumerate() {
                let declared: Stratum = decode(&array[i]["stratum"], name)?;
                if &declared != s {
                    return Err(Error::InvalidReport(format!("reference {name} order {i}")));
                }
            }
        }
        for (i, &(path, bytes, sha, _)) in ROW_IDENTITIES.iter().enumerate() {
            let expected = Receipt {
                path: path.into(),
                bytes,
                sha256: sha.into(),
            };
            for value in [
                &refs.kernel["ordered_row_receipts"][i]["receipt"],
                &refs.game["panels"][i]["row_receipt"],
            ] {
                if decode::<Receipt>(value, "portable row receipt")? != expected {
                    return Err(Error::InvalidReport(format!("portable row metadata {i}")));
                }
            }
        }
        Ok(refs)
    }
    pub fn row_identity(&self, index: usize, rows: &Vec<Row>) -> Result<bool, Error> {
        let &(path, len, _, expected) = ROW_IDENTITIES
            .get(index)
            .ok_or_else(|| Error::InvalidReport("row reference index".into()))?;
        let bytes = canonical_bytes(rows)?;
        Ok(
            u64::try_from(bytes.len()).map_err(|_| Error::ArithmeticOverflow)? == len
                && super::diagnostics::identity(&[(path, &bytes)])? == expected,
        )
    }
    pub fn witnesses(&self) -> Result<(bool, bool), Error> {
        let mut outcomes = true;
        let mut hashes = true;
        let witnesses = self.kernel["witnesses"]
            .as_array()
            .ok_or_else(|| Error::InvalidReport("reference witnesses".into()))?;
        if witnesses.len() != 10 {
            return Err(Error::InvalidReport("reference witness count".into()));
        }
        for witness in witnesses {
            let episode: Episode = decode(&witness["episode"], "witness episode")?;
            let current = run_episode(&episode.config, episode.profile)?;
            outcomes &= current == episode;
            hashes &=
                trace_hash(&current)? == decode::<String>(&witness["trace_hash"], "witness hash")?;
        }
        Ok((outcomes, hashes))
    }
}
