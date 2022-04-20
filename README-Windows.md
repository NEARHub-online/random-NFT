Non-fungible Token (NFT)
===================

```bash
export NEAR_ENV=testnet
near create-account random-nft-2.nearhubonline.testnet --masterAccount nearhubonline.testnet --initialBalance 5
near deploy --account_id tip.tamago.testnet --wasm-file "out/non_fungible_token.wasm"
near call random-nft-2.nearhubonline.testnet new_default_meta '{"owner_id": "random-nft-2.nearhubonline.testnet"}' --account-id random-nft-2.nearhubonline.testnet
near call random-nft-2.nearhubonline.testnet nft_mint '{}' --account-id  balda.testnet --deposit 5 --gas=75000000000000
```

## To show it on paras
```bash
near call paras-marketplace-v2.testnet storage_deposit '{"accountId":"balda.testnet"}' --accountId balda.testnet --depositYocto 8590000000000000000000
near call random-nft-4.nearhubonline.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v1.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --accountId balda.testnet --depositYocto 400000000000000000000

near call --accountId random-nft-2.nearhubonline.testnet paras-token-v2.testnet nft_approve '{"token_id":"1","account_id":"paras-marketplace-v2.testnet","msg":"{\"market_type\":\"sale\",\"price\":\"1000000000000000000000000\",\"ft_token_id\":\"near\"}"}' --depositYocto 400000000000000000000
```
