use crate::*;

impl Contract {
    //Query for nft token by ID
    pub fn nft_token(&self, Some(token_id): Option<String>) -> Vec<JsonToken> {
        
        //iterate through each token using an iterator
        self.token_metadata_by_id.get(token_id.clone().unwrap_or(""))
        //since we turned the keys into an iterator, we need to turn it back into a vector to return
        .collect()
    }
}