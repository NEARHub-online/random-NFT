use crate::*;

/// external contract calls

#[ext_contract(ext_contract)]
trait ExtContract {
    fn owner_nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
    );
}