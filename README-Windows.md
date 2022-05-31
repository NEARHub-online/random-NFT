Non-fungible Token (NFT)
===================

```bash
export NEAR_ENV=testnet
near create-account famdom1.nearhubonline.testnet --masterAccount nearhubonline.testnet --initialBalance 5
near deploy --account_id famdom1.nearhubonline.testnet --wasm-file "out/non_fungible_token.wasm"
near call famdom1.nearhubonline.testnet new_default_meta '{"owner_id": "famdom1.nearhubonline.testnet", "receiver_id": "balda.testnet", "receiver1_id": "nearhubonline.testnet"}' --accountId famdom1.nearhubonline.testnet
near call famdom1.nearhubonline.testnet nft_mint '{"receiver_id": "balda.testnet"}' --account-id  famdom1.nearhubonline.testnet --gas=75000000000000
```

## To show it on paras
```bash
near call paras-marketplace-v2.testnet storage_deposit '{"accountId":"balda.testnet"}' --accountId balda.testnet --depositYocto 8590000000000000000000
near call random-nft-4.nearhubonline.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v1.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --accountId balda.testnet --depositYocto 400000000000000000000

near call --accountId random-nft-2.nearhubonline.testnet paras-token-v2.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v2.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --depositYocto 400000000000000000000
```
# MAINNET DEPLOY
near call famdom1.nearhubonline.near new_default_meta '{"owner_id": "famdom1.nearhubonline.near", "receiver_id": "nearhubcomics.sputnik-dao.near", "receiver1_id": "nearhub-dao.sputnik-dao.near", "receiver2_id": "samtoshi_f_baby.near"}' --accountId famdom1.nearhubonline.near
near call famdom1.nearhubonline.near nft_mint_owner_special '{"receiver_id": "contempt.near", "index": 0}' --account-id  famdom1.nearhubonline.near --gas=75000000000000
near call famdom1.nearhubonline.near nft_mint_owner_special '{"receiver_id": "doucelot.near", "index": 1}' --account-id  famdom1.nearhubonline.near --gas=75000000000000
near call famdom1.nearhubonline.near nft_mint_owner_special '{"receiver_id": "cheese.near", "index": 3}' --account-id  famdom1.nearhubonline.near
near call famdom1.nearhubonline.near nft_mint_owner_special '{"receiver_id": "metaverseradio.near", "index": 4}' --account-id  famdom1.nearhubonline.near