/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, Gas
};
use near_sdk::serde_json::json;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    pub token_minted: u16,
    pub token_minted_users: u16,
    current_index: u8,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:img/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEsAAABLCAYAAAA4TnrqAAAaw0lEQVR4nO2cCZhdVZXvf2ce7lBVt4bMc2UOIQlhljCj2AzPAadufCIoz+5WW/t9Dg0tznbbrQ/1dSuK0i22M9oggoBgEiDMJGSAQOZUpea6t+585vO+fW4llJihkmDL6y/r+1K5wzn77PPfa6/hv9a50pQpU2JOyLhEPgHT+OUEWEchJ8A6CjkB1lHICbCOQk6AdRRyAqyjkBNgHYWohzs0jmPx5w+/kKQ/+FwSx4/zuuM+Nh49+HjHYcyB0iveEyPJcuOejiCHBEtSFOqFAlLdRVGVZPA4ASk6cBMSErH4P37FfKTkS+Jo9BNZatyYeC81vo+jKBkkOZZYTDkZWxwZNU5HlmWiZIyXIUnOjeNkLEmWDrx/eQEbJ0ujr6Nk4mKs5MJEYdSYkiJDFBPJElpTFkWWG+McRg4KljjJKRRZ+cbXIy9dSGGogCyrqIpMGPgQRclqhLGErqu4vk8Ux+ialkw4kCCSJJQ4Ro0idF2jTowaQxBF+BJYUYwiSwSxknymGzqe66BEEbasUItCPFNDjyTi0EclJJYVPElGicTY4BIhSypqHKIoEoQQyiqyAl7gJdilJJNIjqipEaosY7ghKjKeJuFKMc26ya4f/ZJCbx+aZR49WOIqUrVO5uSFxO9+G8U93RSdOgrQZFkYQm1jqEchI66DZZhEUUTBdZMVEmsXhgGGotJq2SiShOvWKboeiqqSMU0MWUn0qei6uEGQABtGEaqqoBgWhqpQqlaohRE5O4UmVl7oXxxTqNdRZQXbMCjWa5iKSotpEMbihiQKtSoeMS2WTckP8XyXMPSJZRlT18kYZgK2aehEnk9852+QPA+OCSxItt5gIU+4twt/cJhVLa1U44iNPb1UJTnZGE2awbJ0hl35Aq2azhw9he95RLpKRjEpV2s8ne+hSsh8q4n5mRx9fo0tvX3URrdwZyrFFFnDjFXSkcEOx+Gxgb04MVzQMZu0Ak8P9FISmig17MvpVgumH/F4vo9lTe14XsCWwW4MVSUMAk5Lt+MaCk/29zKnKcvcyGBqlKUqR2yqVtiwbyeGqpPJZqhUqviOk5idI8mhDbxQHk1hxPdYGcp8Y/FSVNPipmfX8809L2LrKhdnpvDls8/lP9bcSVu6hdcvOgVUGbQY9DRBPs9pv72Tsu/zvxct4px58yGs8/G1v+O7PXuY29TEV+cuYXqqGVrToNiQ97ivtIevr76Xj7ZM4qRlC7nud3fz0OBAotEzMwb/sup1lLZ38f0ul49e+Hp+unUjD/d30W6nExv3N4uXMLs9x+fXrOaDCzqZYEwEvQnSKhQq3LnnBW7YuQUnCjFjCckPjsPAxzGyrCCpKn4Uk5Fi4moeKgp/PnMev+rrYo9bRlElYYRQ3JDYDKnFIXc/9wxbqwXadRPXD/CAaZkWFhgt1J5dj93WzLkTp/H9vq5kK2uOTmwYfOyun7G3PMjHl5zH61ecCjO3URvaCcFkAqlhyIUd1CKQi2X8SoUO3QC3iFOrJvMVjkEXNq1cxbBN5ismEyKb3eUCf/vEHbRrKp9dcgZXnrSSZ0Off9vzIpORDzioYwNLTCyKUHwpsRWOqlKVW9jbt5uls00+2TGDq7c+SyBADEaoajrtWgo0mVu6d7OmMExO0zFVFUnXuDzTRPvUdr64ZS3vpINTZ0+l9UUTL44IlBEi32Z9tZkHCzWCjev5eecMFrRNohy6BMUa5bpDwXexFIUhT2E4DpBVFTXIQqwQRWFig5J5E+P6IbEHpcilrBro+REeK5Tp7clzSqaD981dxFLFRBUOQYoaHvOYwUq0K0qMqXDffhhiGyabd+7CDatcuuB0pvTuIk+AJLyRK6MEYMcSp7ZPoleSmJRK40cRXcUCp6QyIMk80zvCaVqWi+almGVn2FEaQZc1SAuX0IfiDrCyeQlSdgLd5Z0oIyWU6Sqn2in6XIeAmGVGlpShs0uqUQqqEMgEwlmMhjAhMY4aEKvCgah4fkRgx5yZ0UmdsogLFi5g12AXP+jZiWkayK47PrU6bJw16uajJCYB3VDx7RS37u7hlmXNXJtpYctQN+irEiAcHMqFXj6xZCl/Z5xFNmXw4Nbn+cCTj3LqpIn07NvJ3SM9TJ3czEVWhrMybTxXGGZIVskVS9y4eDF/NW8eb5gym7UvbOJzLzzBZxadie85fPzklXxC0yHSIFKplvdRVyIkMwI1wpWlJEwQ6UgQx9SVUBwGikY1DGi2DH7+1vcg6Rmo5Lnx/vt4sDTAwilTkV3v+MFC2h8wipcy1Cv4TU38erif7sFh3jd1Bl9bv5tytYSkykQyyJbBMy9tIx/G5AyN1fl+JrW3MzmX4ekNG5mpasSeA7Ua87LNSJpKPfCRXYUlVhvpea2YvsJjzzzBmpEujPY3UNcjHu3eiePFyKhkVIsFbTaSIqGLYFmKMJI4szFX8VJoueZFZMKYtK5S9ev8++qHMNUU75q3gI9dcBkDGx/nofw+2iXGZdwPC5bYxVEYNlZMgKUq5Ow0Q36d72xfz2eWLmfV0Byq5UJiL/TQxMrN5Ht71nH7YDd62wS8ksPX5ncieQ7zZ8xl48JVqGEZhvJ0NrdwcqSSq3tU26fw5/fegZfVuO30i/nwuVfwbDVPrauL7IRl/GD34/yirzfxhoubctw5/WJa9oFfqIAqDH+EF4dJHCW2oSYpEEYUgzKqEVEpx/xz1x72OtA/MsiNV/4Zn561kPXFYWp+ddxgHTKRFqcritLIIoRrdTzUIKLNtPh5/y42lwtcsnA5hmlQl4Q1iaDmonsBViRjxjG5esQZVgpVsbi3r5dvPP0QN29+jC2ez8JcB69TZcrlIaqaTGnKFFZXqjywrwdTt1nV2kGtXkTCJJ3JYTVlSDVn0Wwb05eQI5UnopDIhSXpZuQoZutgHzndYsmkaQxWq+wqFcFUsOohUmyAnGIkDiAo0hJFSTArMg8pjo5fs8Q2DOMouXnMDFYEtiQzEAbcuncbN89dSlOsIyOj4CFXh/lwZydXxbMJ8VHnmyyf1EY9X+P/7tjKS7URqq6DYbexWFc5ZUIzyC6TophpkcwmTWNjtZjEYsuzrWiSBykNDQk1itFELhgFqFFAc8cEfte9jV/s3MNbp8/h66f6bOjt4a0zVpBqauGfNj7Di7FMlhTlzAh/Mb2DnNnO9QtmgZ7jJwOb6XGq5GSFyrigOixYjQRWlaQkcq8R40nCbsS02SnW9nSzITeBeZM7cUKVsiRR8SvMmDmV+aZNFFTxFRnP93h+sA8HmNzaRslx2VIcpjLQzZKJU0GRKJSHcUOHZt1iUznProFuFrbNwMNlpDBIEQ9NkpNtEMoxFUKiSE+87xe2PUe7CldNmc5V0zqhEvHtjev4Vt9uZmZa2TtcoLUlzefPuyDRgHJ+H//w5Bb+bc8+smkLte4iK+q4+IuD1g1FjEWhyIKPXI9/xSVU+wZJi1QiinBFEi1J+EGQ2BBTN3BEbidLwnyMDhAfyOAVTbhvHyeMGpl+3BjfVmQMTU/SF9f3ku/FGUKTTUlOQhXBTHihTz0JY6RkAWUkUoqS2FLxecF1km3UmW4BRabiO3RXKti6jqVqmIqcxHuG0P3Ap+TU6HNdUpZFOmUTVxyGbvpH6t09KOnUsWlWEIY0pdMoHRMYdgPcMEiy9lZFbVAkEtTDMAkvbMUW+Iikv8EPNdYheS0yAMVO0abIB6gkT4AeRtRH11M2DZplRShZcrZI0EUinNBpWLRoGqrSMNpiXiJTEHPIKRrtkpQwFPnAT+gdSU8xu6kZXRj7KKQWhJQkEsYj1BX0dIolkkwoKCzLIrLrDEXjC0sPCpYIRGXbYueaR5CH+gmLNQxBicQRI2JgRWrwQLGEkUTBUnKOSGLF6iuygiyiaQFqLCeUTFWWUIRXlRrxmzXqsQIpSDQ1T4OTkhKuKkJu0FAogUycy1Ky1UQLKpU6GiaBIeHHEbIb4SsRwiVHQsMVDUfMiSgB2AhjPENCCRQCxcMQdE5dpj+lM2zYMNBLOFJEOQLjcEiwxKqJHKumaXi2JUKZhHpJljpqaFUsN7QnmecoCxEIrmvUMYjYrMHHxaPvk3VvKJywbwJ0sb2kBr2z/5wgDBKwxPZULINq1wClb3wzSUns5YuZ/KYrqfYXQYOyJHJSJdneYpP6oYumK0kyHQYRqmYgCa4sgpIVY0saI4rCU206c6UMc+0Usq6z3VyPWy42tPdowRKghLUa7SuXY7zrzfg9fQ01HWVHG8eM/ie9/GZ/EDuWcHyZ+n0lndr4I41hjxt86Rj2U5bQ23P0z5vN7s/8A83zlzDp2ndQ3bYzOekA+A0adwxrOoZRHR1baKyj6iiyxhLf4aTeLlb1vEQ40+Kr92kMFWJU4xg0i1Fa2a1UiYeGiUaKhx/ljyVxjF8o0nbpuZSe34belSfY+BJe4CIH4UEvGiYkodD4cAx9H1PXLWqqxGlbn+Kza7/P7KiLkuFht1vsmDCDL+3RsA9P+R+adYhF9B4EB7TlTyLi2mGIMzDM9I9eR/jcVmpDQ8hNmZenOqqZjmbiC1spkv8YHF0h5VaRowBXt3EllasfuZObNv0UrBrbI40pPuiun3jx+LgKFmNywz+piDkEIWH/EMyeSlR3kILggDMIFBVXNTBrVVrCGst3PkNzpcC6WafQM3UeSuDiovCZe27lHT33EKXh5twbuXPFxbzt+UeYXs9zx65dNGmiWHL4yuAhbVaiXa8FsGgA5qgGfiBhSAq6SK1GgaroFm3FQT7123/lkpFNeLpOGDhUnr+Dvz3nBu477U1cu/orvL3nwSS5u37Fdfzq9OtpjQ2+OnMB7VGMu/oTmIN5MDKHncZh64bjTTBfbREa4ykqvmqgRj5G6GM5FTKeh2eliARnRMxAOsclmx7lU098nelhibqb4gVrGtO1bbQXQ2aNdBF5Nf7HjieRNJe9chvUOmiqDJAJfOKmdoqq0SipRUeOtA6rd9JBiqn/JUBpesJ0ZMvDIojDlSL+10P/h7t+/Aku23wfI4bNiNXEwq4X+eKj32J6ocSP285ixV98ky+tOJ+2fMhwFh6dsxAjqrG5vRN8mK4N8akXv8H9t17F3/3np1m04V4m1rpAVUVR7YhzO+QRwu0mac8fQbtETCT4dF/Rfs//iM8DVaWopXjjxnXc9stP8uZn7yH0ZVYWt9Eij3Du9oeJ/AC9VuOfHvgqHe4QP1t4KZ97/UfomdGJabTj1Cw22GcymJtDJqzzrbPeyd2TzqTkTKQ1tJih+FzkbODbq7/ANQ/eQhiE+IJcPIIcmfx7lTUrMcqqlhhlzXepqxpW4I16NYm8leXCZx/m4+t+QCpT4JKex3hg1hkMpTspyFtpkhw0ZM7a/TBLS3vpzc7ga6e/A88ymN+3gx0ds7j02q8l1zAjD7Nex1MMbrjkQ/iKymUbfsdbtq+mVdpFZ7rOW156ji/0y/jGkcE6LJ91PN5QgBJJchLfuJqZOIvkM1nGk1WW7niG96y+nfZyvvH9qPtPlUe45vlfkLIKDKgW5diiPzuFvVoz2XqAbIV0lPO8d8PjyezXNbUxbFuk3VpSzRZbuK+llWLKHq0jgB6HCYMqSl73Ljmbj55/Pb3SLBiCHy09F3fhAvRq9Yj3dHgDf4wg+apOXTNRRBmtkseXZep2E3roUTFsVm1cwz8+/nVyXsDyUg8fuOJTxJpBTbc5ZfcGFpS3sy8FTXWo2Qr5XJoNk6dz9V6IvRS6X2WOuy3R/q3NU3DNNOlaPglIHc1IuC/T9xv2T9XxZfjIg7dy+Z4tdE80yVVrtPVW+M7i8/jKpdejdP8Abc0ayBwj68AxalbNsFE9l5Rf4MztT/HWTQ/QbWb4ycor2DprGa6sMrPUSy4M6G+BpYXnWL53A+sWnpFUWs7ftoFMCAXFoCN2yFupBPSh1OQkIS+oaVxFoWSHtNdkVgzt4eflAYZzU9C8KpOG95HzK/Sl2/EVPdl6drnAuQMbaFeLGEWFbq2FT17+l6zvfB1pz6daLFA9nop0vL/laCx4yTZSElUXaq0HPqpIfEc7YTzNwpMU3v70z/jA5nuQDJ+MaOyILM5+5DPcWP1rfn3G1QylbVGvJYokQi3m2g3f45H5K9DjiEsHnsOxLXQ/g+wP0Jdpp2ZlsENZFHLYZc5gX3Y6W7S5zJGf5dLhTex7+HvcvPwqVgzt45bf/TNDKZlrL/si3a2dpL0ykm7w/ss+m7BhIpd1ZJVSJocR+5hC2wVQ47DNh+/P2p8gj9qaqm4l1ixVLSWrXU+lEuOmBT5VK41Udrjpvpt5R20dkqPwUOZk6rrNhe52ZtRrvHfzOh5ddDl7m6Yxoknk/IyIDOgcipjfPYQRlUgrFfJFB980CSyJjRNmJGGE4VeRAqimLCpNzdyy4u1MeKTCMq+b9/av5c2/XktzyaBspPji6VfT3TablFtBjiNiSaY/O6FhtwTtE8ek3JrI04lDA0k0hxxvuiM4KsFCuqqGr5kYYcj8XU9y1ZO/pj2K+OHK83hw4cWIuHfYauGaR27nnd3r6E238jer3sT2k96EVt5N5eHv8raowEm1fky/gBI20VpXqKqiZclBbzG44qmf0+wNMpLO8ZOJZ3Jx7xPM8Fsp5DpRvSqWn0f0ApR0HSN02DFzPh9qvYHrVt/GysILYKb58pKzeWDJRXhGCturJ8n0fq1PebU/VAa5wanh+cfZzBaD73uEhkbVbmLB5if58ppvsyDuJdBNAs1hwdoXKEut3LfyIs7esI6Pvvgrahn4++Xv5/6zLseWQtSO5TzQdT5v2LAVWa4lkzeiMoYU8Mu2SThGE++sP8eFleewnH5q2SncvfJKzv7tVgJ/JEmMBe81rdLojejJtiVana2XqVs237ngGm5z3SRuK2dzyfipxDOGR9YWUdXRVeKUmfBexwxWEIc0ZzvoUybzzt98mRvX346UhRfsRXSpk1lWW09HfZgZQ91oTswHn/oPcuEw/zLpDO5ddhkzqhWane046nRenDifqlvDtTN4sk3a60KuwYOLL2f1rFM588d/yUJnBEeF36g6BbmJJieN5e4mIESXYGdmPnfNKLFx2slk3VJiGiyvjqcZOEYq0R7br6OEDZDGndeK4+TxBQUHjbMEU+maJm3NbbxnzU/52Iu3o6XhhtOuZdV13+S7SxfRXh+mbqg8uHglEwd287rhnZSR+PWiK0ipMbK3k5GUiWtYtAcVJldh9bRTKGkqZ+x9Cs9W2DVxHsXcZG467Toqrofqqvxw2bsYmjiZO2av5OHsAupWho7yEI8sWMXn3/hhqpqJ5TsH5qoHHpZXS4CTBft6NB58lAKSRAn/WLehJKoksYa+ZR3vN9aSCuCGs9/H91e+BV0LcJo66A/S/HDJ2Tw/62Suvv+H6G6dF8xmdk2YjSGVyAUxvfJ08lrEB7fciWvBvy+5hCanyqr+PehamFDFzZU8mzpX8uNd5zE1P8jOjvlMqA5xz7LzeWDxOUSKlgDhaxqBYmF6zoGSyPFLg/7BdY/DwEsyfiwzffdztM4aYktLJ/fOW0VT5NAy2EdPxyze/O6vULNtOkYKLKjuJE5DLiiTK+2jt3UCZXuSKJ9wzpO/5N3b13D/5FnsaZ5EcxARuwojou9UlNJ8Fzv0uPXc/4mcFBwgVa/iqxpVowGOuA8Rooh/r7ok23B8He4HBaseSkzLyLy9YzjJ1n+54GyGmtqYWBsa7SqOybdPxPKcpOj6+NRTeM/2tUzz69z4+I+4rdRPTUvzth2b+Mj6n3HXSady03l/TZO4oBLykYs+RKDI1IwUKb+eXFPwVZFuJWPyCnBePU06hByhhfywYAUiTbENppoOYTGgqMb4uk5cl/AUHU/VsGsVtDhMPMr6mSdzz4Szedvu33Jp6WnmbHya1KBCe03i9hVv4EtCawyLbL2UAP1S59JkdtnqSKMChIQRvNz680cHZ6zEjaLweK548LohUtJBPBwbpNNtXLb9MdbMPovBiZ2kqsPMGNjLJc8/zn+uuDABziTic5e8l7WbZ/LuHY+S8T3umjeHu046j66OeYk3s+vFBgiSRLZWalxIeK0/DgTjFjEDSYqPvchqqzE7RkJ+MWRzw5xBzunexg3338pP513AmfkN/NWm1XTrJncsfV2SNKfrFTzT5qEVl/JM5xkYxIzYWWqmTdopJ2nRa4aiHisixRGtAIKeGQdTenBvKDpn4oDHJq7gWfUFpkjbuLJ3I+f1PI2tKGyzFvL357yZQjaXRMpJIOjUCBWFaiabdKXoYUCukj+6mOdPIKKnKxatVePQrYNXpKOYdjy6J83lmkuuYf4Lmzj/+QeIJZ9HZ5/F1slLqKezpJ0S8pggUMQ5iYHeP85rGKRExPz8IHnkZjwe8eCalcRqEXLgEBk2m6fNY9vEaYktqxoptMAlUxtpsJuvdUAOJ2IbaiqxbSSP2BxJDllkFWop8ivTq6E7FQJFSxQ1Wyv+4YNV/z+LAMkPjy+RFlsxChrpgxhGC/3/DtD8vowWcCXHG9cOOXhumATx8ujzLP/NfyMjieCPowFXGmVJ9z8TeEIOA1Yih3qK9WhF9FppKkpzU8Pj/BcXbccl45zTocGSpFelwCoZOsFgntqzm5BTdvLw1GsOsOPtg3+1NEsyTaJanf5Pf5ny3Q8gd7SBrr82ADsQOhwnU5rcyn4jvz9gk/b/GZumj3m9/1nlsV1+4vHf5kyiUQNf/07SKp658lL8WrXxMMIr5Viq4Mcay4j709TGPR5XdSfJm2SUpiw4+5+cisf2NI49eLSHcvRGD2i1lJwfDg4jZ9LItVoCWKTIZN56WfKsomiYO3DD8VjAD74eB30dxwcW5qDAvXK+o+8lTUueU6RYTnLEI8khfz9LNI2Zc2djzJtNVGtwTPsbcBs1pDGn7b9BoYFjm0lEjGYa+D39OJu3IukqYaWWMLHpc85Atixi3z9wbGP8aIwmv1JjD/E6avRrJY+vSWPmdAAc6eDvRRLtetTXbyRyPCT18IAd+sfGFIWwVCYqlV82ykeslrzimNGbF4ApLc0HJhp7HmGh+PvAHurGxiNjgXzlWK/8/A+0TErmJh4iPVLKc+KX2Y5CTvy8ylHICbCOQk6AdRRyAqyjkBNgHYWcAOso5ARYRyEnwBqvAP8P8+Rap6ZR7BAAAAAASUVORK5CYII=";
const MAX_NFT_MINT: u16 = 500;
const MAX_NFT_MINT_USERS: u16 = 300;
const NFT_IMAGES: [&str; 5] = [
        "https://cloudflare-ipfs.com/ipfs/bafybeiejustedpnpl2sl37dvmifszj6xazi6rc7hdulc744nqtkyii7tdi/WL%205%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeifz7txlqaghmd65xuf3pm6h2sqp2j7szerellxmyxpho74ao7yzcu/WL%204%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeigllxpu5lwak6hilfojc4dssi43pxajhf3lnichxvut6lwf3ekjsm/WL%203%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeigrw46fpw3wldc4jdwwpabuift4nk4egkdqdui5dqidfxdon3vgnq/WL%202%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeie6tdmf5whxd4sy4b7wtnzjafja4pgvlsah2jxknd6wxgtjzqngvy/WL%201%20HRMS%20copy.jpg"];
const NFT_IMAGE_HASHES: [&str; 5] = [
        "3d26f2df03dc554ce08215b208da8047230e350b58784ff94bcc9a24622625f5", 
        "210d372c6d7f08f89478b84c4aabfd9f4991a29198240b06f4102d81dd5bf38d", 
        "cdf42b036d29445986c65d2c271305fa9536738c71249cb53b1682896ee599d6", 
        "87393da5cbdb077e68d4e15a14c423c70e6921cbf5c859792f0ad6a5c7d6b585", 
        "5c9fddd986a2453a482cbd7e541107a023145b7538b6fe0b8c7cbe4fb79dbdfd"
];
const MINT_PRICE: u128 = 5_000_000_000_000_000_000_000_000;
const GAS_RESERVED_FOR_CURRENT_CALL: Gas = 20_000_000_000_000;


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Near Hub NFT Comics".to_string(),
                symbol: "NHCOMICS".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            token_minted: 0,
            token_minted_users: 0,
            current_index: 0,
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
    ) -> Promise {
        assert!(
            env::attached_deposit() >= MINT_PRICE,
            "Attached deposit must be greater than MINT_PRICE"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        assert!(
            self.token_minted_users < MAX_NFT_MINT_USERS,
            "Max token on sale is MAX_NFT_MINT_USERS"
        );

        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GAS_RESERVED_FOR_CURRENT_CALL;
        Promise::new(env::current_account_id()).function_call(
            b"owner_nft_mint".to_vec(),
            json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }

    #[payable]
    pub fn owner_nft_mint(
        &mut self,
        receiver_id: ValidAccountId,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only the contract owner can call this method"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        if self.current_index > 4 {
            self.current_index = 0
        }
        let url = NFT_IMAGES[self.current_index as usize];
        let l: String;
        match self.current_index {
            0 => l = String::from("a"),
            1 => l = String::from("b"),
            2 => l = String::from("c"),
            3 => l = String::from("d"),
            4 => l = String::from("e"),
            _ => l = String::from("e"),
        }
        let title: String =format!("HRMS #1{} Whitelist NFTs", l);
        let _metadata = TokenMetadata {
            title: Some(title.into()),
            description: Some("NFTs created to participate in the whitelist portion of the NEARHUB Comic issue #1 PFP NFT drop.".into()),
            media: Some(url.to_string()),
            media_hash: None,
            copies: Some(100u64),
            issued_at: Some(env::block_timestamp().to_string()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };
        if self.current_index == 4 {
            self.current_index = 0
        }
        else{
            self.current_index += 1;
        }
        self.current_index += 1;
        self.token_minted += 1;
        if env::current_account_id() != env::signer_account_id() {
            self.token_minted_users += 1;
        }
        self.tokens.mint(self.token_minted.to_string(), receiver_id, Some(_metadata))
    }

    pub fn get_minted_quantity(&self) -> u16 {
        self.token_minted
    }

    pub fn get_user_minted_quantity(&self) -> u16 {
        self.token_minted_users
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
