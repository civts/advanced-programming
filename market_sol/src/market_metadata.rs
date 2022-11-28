use crate::good_meta::GoodMeta;

use std::collections::HashMap;
use unitn_market_2022::good::good_kind::GoodKind;

pub(crate) struct MarketMetadata {
    pub goods_meta: HashMap<GoodKind, GoodMeta>,
}

impl MarketMetadata {
    // fn get_mut_good_meta(&mut self, good_kind: &GoodKind) -> Result<&mut GoodMeta, MarketError> {
    //     if let Some(good_meta) = self.goods_meta.get_mut(good_kind) {
    //         Ok(good_meta)
    //     } else {
    //         Err(MarketError::GeneralError("Good Kind not found".to_string()))
    //     }
    // }

    // fn get_mut_good_meta_from_token(
    //     &mut self,
    //     token: String,
    // ) -> Result<(&GoodKind, &mut GoodMeta), MarketError> {
    //     for (kind, meta) in self.goods_meta.iter_mut() {
    //         if meta.token == token {
    //             return Ok((kind, meta));
    //         }
    //     }
    //     Err(MarketError::GeneralError("Token not found".to_string()))
    // }
}
