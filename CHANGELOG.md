# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Features

- Add CHANGELOG.md ([c572afb](c572afb13849756b27394bbd32dda4e99efc9e0a))

### Miscellaneous Tasks

- Other is '^.*' only ([3005a5a](3005a5ad9708c69baedd0b5cfa147cb49fd2ef1c))
- Add commit_ids to changelog ([e5d9e64](e5d9e64f9b822f562fab5897a91db9c36a217b24))

## [0.6.2] - 2022-04-07

[2df56c3](2df56c309fed733fc2df8887c20581675cd28295)...[21f7fb7](21f7fb7b6e7ad34a3d0e5ebd36d053a3d3a50e46)

### Documentation

- Update README.md ([4d161d4](4d161d4d91c3cd22daa9a1be9a9dd6453904e220))

### Other

- Bump v0.6.2
 ([21f7fb7](21f7fb7b6e7ad34a3d0e5ebd36d053a3d3a50e46))
- Lint failure ([53042bc](53042bc715428b97b4d38f77ef56d11ac14e20ec))

## [0.6.1] - 2022-04-06

[899e51a](899e51a612bce6c7e04c3ac3a55a1d1010633461)...[2df56c3](2df56c309fed733fc2df8887c20581675cd28295)

### Other

- Cleanup scripts, bump to v0.6.1

Discovered bash cheat-sheet:
  https://bertvv.github.io/cheat-sheets/Bash.html

Used it and shellcheck to cleanup the scripts.

Also, output `*.result.txt` files so they manually be compared and verify
that the quantities and assets didn't change while transforming. Right
now its pretty easy to manually do I just use `vimdiff` and look at the
difference.

Such as `vimdiff data2/b.us/b.us.q1*.result.txt`. The minimal check is to
verify that the "Total quantity: xx,yyy,zzz" is identical on all three.
If they are not identical then compare the Asset and Quantities (ignore
the Txs count column) and find which ones don't match. You will find
a few which aren't identical but are numerically the same as the
difference will be trailing zeros. Look for actual differences.

I've created [issue #11](https://github.com/winksaville/rust-binance-cli/issues/11)
to provide a better way to validate.
 ([2df56c3](2df56c309fed733fc2df8887c20581675cd28295))

## [0.6.0] - 2022-04-05

[6224b8f](6224b8fd2f364aac76b81436de6ba6aad8973b3e)...[899e51a](899e51a612bce6c7e04c3ac3a55a1d1010633461)

### Other

- Bump version to v0.6.0
 ([899e51a](899e51a612bce6c7e04c3ac3a55a1d1010633461))
- Add some shell scripts

 - filename.sh processes a filename with path from the command line
 - b.com.cttf.sh converts binance.com trade history file to a
   tokentax CSV file where the income records are consolidated into
   monthly buckets.
 - b.us.cttf.sh converts binance.us distribution file to a
   tokentax CSV file where the income records are consolidated into
   monthly buckets.
 ([818a61d](818a61dc00c7eb9399db5677c23d069b02898114))
- Some cleanup

Found these while processing some data for one quarter.
 ([8bcfb64](8bcfb64bae40de69515f850f1d7e4dc09eb401cb))

## [0.5.0] - 2022-03-25

[f9b91a4](f9b91a48040b34e3fd07633a3b8fff1b47a17e7c)...[6224b8f](6224b8fd2f364aac76b81436de6ba6aad8973b3e)

### Other

- Upgrade & update deps, version to 0.5.0
 ([6224b8f](6224b8fd2f364aac76b81436de6ba6aad8973b3e))

## [0.4.0] - 2022-03-24

[aa97e9a](aa97e9aa9d3cc1d0550a7ab66b800fff670551d7)...[f9b91a4](f9b91a48040b34e3fd07633a3b8fff1b47a17e7c)

### Other

- Bump version to 0.4.0
 ([f9b91a4](f9b91a48040b34e3fd07633a3b8fff1b47a17e7c))
- Use taxbitrec module.
 ([5af2f6c](5af2f6ccabb3a66fdec316e8e1178d1c100e04fa))
- Use the tokentaxrec module
 ([09cde89](09cde89681d53d1cb43d69540658ff527d1498ce))

## [0.3.18] - 2022-03-23

[a04e2fe](a04e2fe846f557c6760a66ed9dd42e435617bbe3)...[aa97e9a](aa97e9aa9d3cc1d0550a7ab66b800fff670551d7)

### Other

- Add initial TaxBit support and bump version to 0.3.18
 ([aa97e9a](aa97e9aa9d3cc1d0550a7ab66b800fff670551d7))
- Use dec-utils which was refactored from common.rs
 ([574e6d2](574e6d2ef30a51cf1941a50f5db4ed9ab023ddee))
- Don't use +nightly when executing tarpaulin
 ([c973c73](c973c73a59476cad2e777e7aeaab44f70eccdc22))
- Changed ucttf command to prefer Income records.

These are simpler and for b.com and b.us we do find an "Income" type
so this make the "*.uc.*" files consistent.
 ([efb26d0](efb26d06ddb511c82b115e16105f4a21a9187781))
- Changed TaxBitRec to use se_time_ms_to_utc_z_string for serialization.

This now meets TaxBit's specification. So now we'll output a trailing "Z"
instead of "+00:00" although in actuality TaxBit does support the
"+00:00" just fine. But there documentation doesn't say that so tech
support thinks the format it wrong.

We'll see if they complain about me using milli-seconds but they use that
format in their export format, so hopefully they'll accept it.
 ([185d95d](185d95d5814863d443a1633eb2ebe93277f30768))
- Updated dependencies

So clap::App is deprecated and is the preferred name is clap::Command.

Also, rust_decimal catches a failure I found at runtime it now catches
it at compile time.
 ([191b50d](191b50df6b33b4ebc3e3d6402fda4b6d7ec63741))
- Remove fees which are zero.

TaxBit doesn't seem to like them and drops the records.
 ([e8f10b3](e8f10b3213ba3098cc91a062180485265c24425e))
- Fix broken test in TaxBit
 ([0dfed5a](0dfed5a8b1d3841304bb2d1190599960da79966d))
- Fix clippy warnings on nightly

Initially it was more complex, but this is obviously better code!
Thanks Clippy :)
 ([eda682d](eda682d2f038bc6f3235f27917462941d66d4a11))
- Improve TaxBit record creating

Use Buy and Sale when using USD to buy sell coins.
Use Binance for binance.com exchange.
Use BinanceUS for binance.us exchange.
 ([206664d](206664d0e57285bcd47150cf11d312aa9fd52ee5))
- Use de_stinrg_to_utc_time_ms and se_time_ms_to_utc_string

To serde the time for TaxBit. Also map binance.com and binance.us
to Binance as the "sending_source/receiving_destination" field
contents.

This seems to resolve issues with being unable to import records.
 ([2115831](2115831800afcc6cce0a36df3afaf57527ba3b92))
- Add uniq_currency_token_tax_file
 ([3e0fa2d](3e0fa2d3ba37a48184b5407cb4428de7cb4d45ce))
- Add ptbf and tbffttf commands ([e2a53be](e2a53be1c454496c0ea8b2e553f7e6091583d2f2))
- Improve ci-stable.yml code-coverage generation

Update changed to use a container rather than install tarpaulin from
sources, this should save about 3min of time. At least that's how much
it saved on serde-utc-time-ms.

Also, add ci-stable and ci-nightly badges.
 ([a527503](a527503edb678e4fe4665cdc77956d644ae5a1bc))
- Extract de_string_to_utc_time_ms routines

They are now in crate serde_utc_time_ms.
 ([725144a](725144ab93db4990b5cea65be03a927120c5629d))
- Extract time conversions to it own crate

The crate is time-ms-conversions, currently only in github.com/winksaville.
 ([48b6af3](48b6af3ee7cbe5cf4a41cb710c5b92187e24aeff))

## [0.3.17] - 2022-03-10

[eb3701d](eb3701dda0b8db49793e3a48116046c70982e615)...[a04e2fe](a04e2fe846f557c6760a66ed9dd42e435617bbe3)

### Other

- Bump version to 0.3.17
 ([a04e2fe](a04e2fe846f557c6760a66ed9dd42e435617bbe3))
- Add create_tt_cmt_xxx_string routines

Make it impossible to create an incorrect string. To do this I had to
make DistRec, TradeRec and CommissionRec public along with their fields.
 ([da58fb0](da58fb006b66a24674efd9f32415bcc63fa39bc7))
- Track file_idx and line_number per record

This will make them always correct and consistent. Also added
TT_CMT_VER3_CSV_HEADER which includes the FileIdx and deprecated
TT_CMT_VER2_CSV_HEADER.
 ([7f03e37](7f03e370c7716409dc889667a6da0de46e90154e))
- Handle missing fee by adding fake fee
 ([68142c5](68142c54079a917751c9b3f92b0821b7ff947e35))
- We need to track the line_number for Trades

Accomplish this by adding line_number to TradeRec but have serke skip it.
 ([644852f](644852f4bbc6607f6a6c51d0d42531416c711dca))
- Add handle creating token tax trade record.
 ([06ef6d5](06ef6d530aa7953c66a42d314cdb25dac7a087ab))

## [0.3.16] - 2022-03-09

[798fb7a](798fb7a5d30f7b56edadc74618ae35d9d229d62a)...[eb3701d](eb3701dda0b8db49793e3a48116046c70982e615)

### Other

- Bump version to 0.3.16

This was version was used to process the data and upload to TokenTax
on March 7 2022 after 16:45 PST.
 ([eb3701d](eb3701dda0b8db49793e3a48116046c70982e615))
- Cleanup binance us user output

And be sure data is sorted before calling write_dis_rec_vec_as_token_tax.
 ([0df3115](0df3115fa855893dd6ef9773fbf36d7fb0b416b4))
- Fix bug where coin had no fiat price on TokenTax

I was using the start_period time but this doesn't work if the coin
didn't "exist" at that time. Instead I'll use first_dt, alternatively
we could use the last or sometime between the first and last.

In addition set cur_ttr and first_dt when we end the period and
if it's the first TypeTxs::Income transaction.

Also removed {consolidate_}push_to_consolidated_ttr_vec and just duplicate
the code since it makes it easier to see they are the same. Note we
couldn't use push_to_consolidated_ttr_vec inside the loop because it
causes E0502 compile error.
 ([04505a4](04505a43d8c26738721981db9499563a202bc64a))
- In ttffbcthf sort before writing the TokenTax records
 ([7f180e6](7f180e68d42e798a3eb0ef619067a52024b318dd))
- Fix up from_trade_rec comments
 ([797a87a](797a87a27657dd985bc88768f97e0954040f37bd))
- Add --no-usd-value-needed to process_token_tax
 ([5efff76](5efff762a50837fbd04927bee3cbe52016c8e790))
- Unify processing of a trade.

Place Spot "Buy, Fee and Transaction Related" operations in to one
arm of the match below the documentation.

Tighten it up the code with more asserts.

Cleanup Read files.
 ([6fb232f](6fb232fd1b550516a57d681d49c5f942b81ceb2c))
- Refactor TokenTaxRec::from_trade_rec and process_entry

Use match tuple rather than nested single strings.
 ([0990553](09905539d40426e30828c265063b4f3c0b6da89b))
- WIP Add command ttffbcthf

Create a TokenTax file from a binance.com trade history files.

 - Fix bug in TokenTaxRec::from_trade_rec the amount's needed to be None
 ([7a82ed4](7a82ed484d3c7e392655a6c6906de6e4ddc6ee14))
- Some reoriganization
 ([c0f090b](c0f090bc5d01306c88064b6757bafdf79927ad48))
- Some refactoring

 - Add fn process_entry
 - Add refactor consolidate_binance_com_trade_history_files
 ([057fcff](057fcff1a4aecffd2cc51a844d4d8a60c1b49cda))
- WIP TokenTax::from_trade_rec
 ([a93fe02](a93fe022b7f26d5197fe6a072ae2923fa88d7031))
- Add support for Small assets exchange BNB in from_trade_rec

Also added a test.
 ([175c67a](175c67ac37968036d41ab575079b3e9c71b17168))
- Update process_binance_com_trade_history_files

It is now similar to proces_binance_us_dist_files execpt we're not
looking up the USD values. It doesn't seem important as:
 - b.com doesn't provide that info in trade history anyway
 - It "slows" things
 ([5183f03](5183f0324e2e9eb69fab8470fa4497117c5420b7))
- Cargo fmt

And captialize T in "Total txs count"
 ([cdce982](cdce982df96b91582b1ffb1b3483dd7e6777d5ce))
- Rename xxx_processing.rs files to process_xxx.rs

This is so the sort together.
 ([211a93f](211a93f62c76b8f1710816b404cb9bdbb18814ef))
- Skip and not run any of tests in cli.rs with tarpaulin

Hopefully this will allow github Actions to be more reliable.
 ([69657ac](69657ac781c87fb7ff3a7eef1911cfbd8c8b4b31))
- Change sort order for TokenTax records

Now the currency names sort before they currency amounts, this yeilds
a more natural order, especially with a month of incomes sorting together.
 ([f945e29](f945e2942c04a890aeb50a9537af480440fb20d1))

## [0.3.15] - 2022-03-02

[bd6e548](bd6e548c50513eec6d18027542d2abdab601c5b1)...[798fb7a](798fb7a5d30f7b56edadc74618ae35d9d229d62a)

### Other

- Bump version 0.3.15

Processing and consolidating TokenTax files is working.
 ([798fb7a](798fb7a5d30f7b56edadc74618ae35d9d229d62a))
- Add --no-usd-value-needed

In binance.us some distribution records do not have USD values, use
--no-usd-value-needed to ignore this. If they are needed you can use
ubudf to get missing USD	values. Although, this can be slow and you may
need to use the --throttle-rate-ms from the default of 500ms to something
higher if binance.us starts blocking you.
 ([15524e8](15524e81f7bd0088d1e3aab667831f81417ca029))
- Fix bug in next day calculation

Have both binance_us_processing and token_tax_processing print the
value_usd values the same.
 ([d2cae5b](d2cae5bc125f0d3c14776b00e8e5f99391eb9cfb))
- Use config.progress_info

This will make the output cleaner if --no-progress-info is on as only
println with the value_usd will be printed.
 ([71dead7](71dead7137c81d0594f0a67498ea7cdd06e5b3d9))
- Tweak the update value code

This is so we can see which line is being updated in the verbose output.
 ([df78356](df783560dd82cc57939a26d8b9a84b75589f142b))
- Add throttle-rate-ms option

Used by convert to hopefully not abuse the service. It
defaults to 500ms
 ([5806f72](5806f7257d015c0b3813f4e06f978d1a694ee468))
- Use progress_info to set leading_nl
 ([067729e](067729e332cd953d3951f2391b52504208cd83ae))
- Sort the ttr_vec data before outputing

This is what I'm doing in binance_us_processing so the same should be
done here. The reason was that in binance_us_processing the input files
can be in any order. Although actually processing multiple files isn't
typical it could happen and there is no real guarantee the input data
is in order so this is a good idea.
 ([cbb0af0](cbb0af0d5c1b5799fb5c65a972042ace26a8b14b))
- Change computing of the USD value of the assests

After processing use a date of the next day of the last record instead
of now. This will make the value consistent and is the obvious choice and
it's what I'm doing in token_tax_processing.

fix binance_us_processing formatting
 ([47c676e](47c676eae29b3115bab5d7eca1049cb1ecd1bf1d))
- Add global args progress-info and no-progress-info

When capturing the output at the command line with '>' or '| tee ..'
the first line contains all of the progress-info output. On large
inputs this can be huge. This adds --no-progress-info so it can be
turned off and --progress-info to turn it on. It's on by default.
 ([1be3dcd](1be3dcd87969636f963484805304dd0fdc2ef0fa))
- Fix end_period_ms bug.
 ([7587fa3](7587fa3718b9515c77aab95e05bf4f55284ece0c))
- Tweak binance_us_processing

 - Changed process_binance_us_dist_files so it saves the data
   in data.dist_rec_vec and after "processing" the data is sorts
   it before writing it out. Previously I didn't do that as the files
   were read in sorted order. We shouldn't assume that.

 - Changed get_asset_with_line_number to get_asset_only because while
   reading unprocessed files there are missing values and using
   get_asset_quantity_usd_value results in panic's. Whereas get_asset_only
   will only get the "base" or "primary" asset and nothing else.

 - Changed get_asset_in_usd_value_update_if_none to use convert instead
   of get_line_of_primary_asset_for_value_asset. Also, renamed asset_value
   to quantity.

 - Changed pbudf to use convert instead of
   get_asset_in_us_value_update_if_none
 ([a9081cd](a9081cd4a4b1af79fb2b254e398c453a6971f04a))
- Print which file names

Also improve get_asset_quantity_usd_value line_number output and added
get_asset_with_line_number so on errors while processing we have the
extra information.
 ([0cae3d2](0cae3d253a8ee16d6b7397f3d46ddd8d1342f784))
- Adding cttf and rename token_tax to token_tax_processing

Running cttf on x.tt.csv generating x.tt.cttf.csv and then running
pttf on x.tt.csv and x.tt.cttf.csv yields identical results.

Change default for Keys to be something so invoking get_klines will
work even if no user configuration is given.

Fix test failures caused by not default keys to None.
 ([6881faa](6881faa4b36de8ad7da166106745bb982ed99d8b))
- Add date_time_utc module

It wraps a DateTime<Utc> and provides some new associated functions.
 ([21adbc9](21adbc966109063b71fe121652063b6b19ac2675))
- Removed unused code in process_token_tax_files

Some day I may do something similar but for now it's not necessary and
the commented out code is clutter. If I want to implement something like
that I can just it from binance_us_processing.
 ([2d49b98](2d49b98eadc691206fbf0c5d5ede5b94ea0ff058))

## [0.3.14] - 2022-02-25

[e53e48f](e53e48f015dc2c06f48a429b57ed2f24779a2c15)...[bd6e548](bd6e548c50513eec6d18027542d2abdab601c5b1)

### Other

- Bump version to 0.3.14
 ([bd6e548](bd6e548c50513eec6d18027542d2abdab601c5b1))
- Add get_asset_quantity_usd_value

The get_asset_quantity_usd_value takes into account the category and
operation when deciding what asset is the "asset" being more consistent.
In particular now when selling the quote_asset is the main asset instead
of the base_asset for both buying and selling.

This make the transaction_count field for each asset the same between what
is reported by ttffbudf and pttf. It isn't super significant as the
main thing is the quantity didn't change.

Also some tweaking of names.
 ([2ac5535](2ac55358a19a6f50858fa19f8c0ae51d9877ee7e))
- Actually process the entires

We are processing and summing the fields in AssetRec's.
 ([a6a178d](a6a178d53fa80cbb869220048c2b8bfc508a8410))
- Add pttf command

This is the initial implementation of processing token tax files, it
doesn't do much at the moment, but does read the input file and
can write the output file. It also goes through the motions of
displaying the value in USD of the assets.
 ([57420d9](57420d9dd65fd1f35a4267f9e8c6df9cd8086696))
- Improve convert and add new tests

And refactor the value_assets to a lazy_static shared by everyone as
VALUE_ASSETS defined in common.rs.
 ([b2a7b60](b2a7b609089e3c64c1e4f8ab28c6dbea8d3ed7ea))
- Fix bug: allow writing when using pbudf command

As it was only when ubudf had an output file.
 ([a95613e](a95613edadaebe4a63f605204e253b5a38c2e8cc))
- Add tt_file_from_binance_us_dist_file
 ([1642b70](1642b70218331a08aa337dfea471aa1f111c932b))
- Add --time-offset parameter to cbudf and cbcthf

Allow the time field of the records to be offset by some number of days.

Tweak arg_matches
 ([2b520a7](2b520a78a8ae763841a917e6315d20b81c875cb0))
- Add Proof for convert_test
 ([2f19c35](2f19c352435bfa528fd66bca70fb7a3b473ce2a5))

## [0.3.13] - 2022-02-21

[d2a0c23](d2a0c232bd7fbe3865f58cbe392e7b1e03af022c)...[e53e48f](e53e48f015dc2c06f48a429b57ed2f24779a2c15)

### Other

- Bump version to 0.3.13
 ([e53e48f](e53e48f015dc2c06f48a429b57ed2f24779a2c15))
- Refactor binance.us consoldiate distribution records

Also, changed to using dr.time of the first entry rather than
the last entry. This means the time of the consolidated record
won't "move" relative to other transactions.

This might change in the future :)
 ([20f69fc](20f69fccc81e93479b81b8c554a6e13612ce640d))
- Teach convert to handle any order for asset and other_asset

With this we can now "properly" test convert as we can assert:

  ((convert(A, Q, B) * convert(B, Q, A)) / Q^2) approx_eq 1
 ([95d7e91](95d7e912808c61d3cf6a009559690ac9c3ce220d))
- Improve convert

Allow it to determine the value of any asset-quote using the USD value
for the pair as an intermediate value.
 ([d5f82e0](d5f82e0e4d807088c816e3c4dd8a9f920a134897))

## [0.3.12] - 2022-02-20

[a6af8ed](a6af8ed17adb3eae39939e151dc2cc4dab916f1f)...[d2a0c23](d2a0c232bd7fbe3865f58cbe392e7b1e03af022c)

### Other

- Bump ver to 0.3.12

Merge Add-binance-com-processing-take-2
 ([d2a0c23](d2a0c232bd7fbe3865f58cbe392e7b1e03af022c))
- Add an optional OUT_FILE to pbcthf command

This allows me to generate a single file from mutliple raw binance.com
csv files and specifically I validated that transactions in floating point
notation are converted correctly:

raw:
   36757189,2020-05-13 03:49:51,Spot,Commission History,BTC,5.2E-7,""

pbcthf:
   36757189,2020-05-13T03:49:51.000+00:00,Spot,Commission History,BTC,0.00000052,
 ([976245b](976245b2798d6d0735527320a31e330e4e1efd21))
- Consolidation is "working"

Here we run pbcthf over the trade history:
 $ time cargo run --release pbcthf --no-verbose -f data/b.com-trade-history/2020/part-00000-75d80281-7110-4cdc-9c9a-3a68ebd669b3-c000.csv data/b.com-trade-history/2021/part-0000* | tee b.com.pbcthf.result.txt

And then run cbcthf generating b.com.cbcthf.csv over the same
data as above and then run pbcthf over the consolidated results,
b.com.cbcthf.csv:
 $ time cargo run --release cbcthf --no-verbose -f data/b.com-trade-history/2020/part-00000-75d80281-7110-4cdc-9c9a-3a68ebd669b3-c000.csv data/b.com-trade-history/2021/part-0000* -o b.com.cbcthf.csv | tee b.com.cbcthf.result.txt
 $ time cargo run --release pbcthf --no-verbose -f b.com.cbcthf.csv | tee b.com.cbcthf.pbcthf.result.txt

When we look at the two results.txt files the quantity data is the same,
which is the expected result and the number of records was reduced
from 2.6M to 36K.
 ([fc39332](fc393327ea5241a4f0bf95038e6dad3798c3fa4b))
- Cleanup references to binance.us "dist"
 ([c354870](c35487023a1ef3e7b93e44ff1b97b3ae3622e8fc))
- Use sort_by using ttr_cmp_no_change_no_remark
 ([e87bdd6](e87bdd605c5fc4c6d7f14e57bbb35cc8ffa950e8))
- Manually implement PartialEq, PartialOrd and Ord for TradeRec

The reason is we don't want to use change or remark when doing the
comparison.
 ([30d3ce1](30d3ce162b7b989c9c872abecea168cac59688f5))
- Add cbcthf command

This invokes consolidate_binance_com_trade_history_files but, ATM, it
it does no consolidation.
 ([105ccb4](105ccb4dad5c40e88bcf58b356acada1b953f6a3))
- Add some processing

Add BcAssetRec and BcAssetRecMap and produce the list of assests with
transactions and some totals.
 ([db72820](db728209f4f0b37a8168c744780688d206b3c37f))
- Remove unnecessary deref
 ([9acdd76](9acdd76b3dec901b6bae40ff8305d7409de43658))
- Add code to read the trade records
 ([c05a2e7](c05a2e7fce4d6e2c8d282345c7467d485cdc7f36))
- Rename ProcessedData to BuData
 ([3b46c92](3b46c920eaf25540906da793a67db79029ba54ac))
- Add process_binance_comd_trade_history_files

And rename the other processing_xxx commands too.
 ([d9b5562](d9b5562ee43d2fe186808f5197737f19963efc0e))
- Move verify_input_file_exist and create_buf_writer to common.rs

This is so we can use them in binance_com_processing.rs.

Also added create_buf_writer_form_path as well as create_buf_reader
and create_buf_reader_from_path.

Then use them in binance_us_processing.rs.

Clean up Error and std::error::Error in common.rs.
 ([444055c](444055c7d753b250973b6a07e26765c5c321a306))
- Add binance_com_processing

At the moment it does NO processing it just a couple tests. In particular
create_tt_trade_rec dos nothing expect return the TokenTaxRec as created
by TokenTaxRec::new().
 ([1d89e67](1d89e670199f5b61eb83b80e33f26791c1440d61))
- Add module token_tax_comment_vers.rs

This is so the comment versions are know globally and are unique.
 ([a8292a9](a8292a9329e78151487e3a3d828607ef5fb483d0))
- Move TokenTaxRec::from_dist_rec to binance_us_processing

Move test_dist_rec_to_serialized_token_tax_rec to binance_us_processing as
test_deserialize_binance_us_dist_rec_to_serialized_token_tax_rec

Make token_tax.rs TokenTaxRec fields public.

Make binanace_us_processing.rs DistRec, AssetRec and ProcessedData private.
 ([ee64bdb](ee64bdbbc3ca2f5cd388559845a75dcd0cca411f))
- Rename binance_distribution_processing

To binance_us_processing in preparation to adding binance_com_processing
 ([b135cad](b135cadb00be42415c40e48ba66ff24265f5b686))

## [0.3.11] - 2022-02-12

[8ce6a58](8ce6a5871772cfd3fb1a2d2abb13471db5131c69)...[a6af8ed](a6af8ed17adb3eae39939e151dc2cc4dab916f1f)

### Other

- Bump version to 0.3.11
 ([a6af8ed](a6af8ed17adb3eae39939e151dc2cc4dab916f1f))
- Change the cdf command to output xxx.cdf.csv and xxx.tt.cdf.csv

Plus some clean up
 ([47cec3e](47cec3e3ebc1735bb8b248dd4b44c72d108d4800))
- Fix bug in Token Tax Deposit

Also, make Exchange "binance.us".
 ([5c730cf](5c730cffbe5fc9cf7dd85ca2e9415f804fe89ce6))
- Add write_dis_rec_vec_as_token_tax

Make it the default for the moment.
 ([9f98a7a](9f98a7ad4b19a4ed7b76f1d18a5055a766f7ced5))
- Test dist_rec to serialized TokenTaxRec

- Rename TokenTaxCsvRecV1 to TokenTaxRec
 ([cb71beb](cb71beb098f0c4d103b0ca3deb2faa2917cdb60e))
- Initial token_tax support

Just beginning, not yet functional.
 ([805fe21](805fe21af079b9dba13f32cfa877baa0590b7a11))
- Add deserialize and serialize tests of DistRec
 ([1ae4144](1ae41440b09a8c1781f784d5db38176b9792b6ac))
- Fix handling leading/trailing spaces ft_str_with_fmt_str_to_utc_time_ms

Added using trim()
 ([8bf4a15](8bf4a15caf965e4d530244eb17060419ce14c266))

## [0.3.10] - 2022-02-10

[13f6a48](13f6a48aabc3c1da856ac1a0b67d389c14f71f9a)...[8ce6a58](8ce6a5871772cfd3fb1a2d2abb13471db5131c69)

### Other

- Bump version to 0.3.10
 ([8ce6a58](8ce6a5871772cfd3fb1a2d2abb13471db5131c69))
- Fix bug in dt_str_to_utc_time_ms not handling negative time zones

Add more tests
 ([cfea98a](cfea98aa3676ac53aefd1d7df9555bfac0108e46))

## [0.3.9] - 2022-02-09

[6d5864d](6d5864dfc1384cc130e801aef83545a6c97a9572)...[13f6a48](13f6a48aabc3c1da856ac1a0b67d389c14f71f9a)

### Other

- Bump version to 0.3.9
 ([13f6a48](13f6a48aabc3c1da856ac1a0b67d389c14f71f9a))
- Teach it how to consolidate distributions within a time window

Consolidating distributions to a daily granulating. Previously I
consolidated sequential sets of the same type of Distribution into a
single entry. This could same many days. The new algorithm consolidates
those sequential sets into a series of daily transactions.
 ([5ea27ec](5ea27ec328c97d12368948ab2ae7437e4ea92c45))

## [0.3.8] - 2022-02-07

[d5467b1](d5467b1e3767412a22c14e7f6587ad5887b02910)...[6d5864d](6d5864dfc1384cc130e801aef83545a6c97a9572)

### Other

- Bump version to 0.3.8
 ([6d5864d](6d5864dfc1384cc130e801aef83545a6c97a9572))
- Enhancements

 - Add more states to the state machine so we consolidate the 3 different
   types of Distributions separately.
 - Add a consolidate_x routine to share code.
 - Sort the ProcessedData::consolidated_dist_rec_vec and output it.
 ([a8b8606](a8b8606945c597c253635d53320fd6c0c6de19d7))
- For debug output USD ar.dist_rec_vec and consoldiated_dist_rec_vec

Consolidating there are only 7 see why
 ([4934cf4](4934cf4c7185d823da10fbb148609bacd09ede03))
- Add some helper functions

 - verify_input_file_exist
 - create_buf_writer
 - write_dist_rec_vec
 ([fe5000a](fe5000aae72a7a0257511e1d3fb6f437fca2c982))
- Use DistRec.get_asset, get_value and get_value_usd
 ([78e927f](78e927fc68d29a4a5080512bf1ac7d5a8d821539))
- Added consolidate_distributions

Not sure it's working but did consolidate 4,414,234 to 5280.
 ([00d87c6](00d87c6ff8b99d86b7a3291a22289865e8d2aafa))
- Add DistRecs to AssetRec and ProcessedData

 - Remove3d add_to_dra and add it explicitly in consolidate_dist_files
 - add_dr adds DistRec to AssetRec in AssetRecMap.
 - Print the number of DistRecs for each asset
 ([e259786](e259786d68b370f68bf60abde5bfc2192b58b123))
- Initial consolidate_dist_files

Just copies input to output
 ([f7deb78](f7deb7847ee8439850eefcc31cfc5af601c29de9))
- Add Consolidate

Does nothing yet, need to decide consolidate parameters.
 ([56345d9](56345d9f9ab1788518094eee2d17aa61ebcf9f04))
- Support multiple input files for udf and pdf
 ([c2abb18](c2abb183b5d7d67e4eba7bed624dc5d35b987651))
- Rework pdf output

Add additional information and make it multiple columns and also output
the list "Distribution Others" as I don't quite understand it.
 ([24f478f](24f478f2ba2a188b78afcaf045e8a9097e7c7dfb))
- Rename line_index to line_number

And in the process_dist_files change the line_index to rec_index and
calculate line_number. Should have done this a long time ago :)
 ([de5e001](de5e0018133b41c4f253f08a64c4607635c1f29a))
- Tweak pdf

Make the names and where they are displayed more consistent.
 ([e13266c](e13266c869a36b88a4b4728ea7b22c9abf001071))

## [0.3.7] - 2022-02-03

[6c739e1](6c739e13f5e49111fff92ed680567c1b17911ed7)...[d5467b1](d5467b1e3767412a22c14e7f6587ad5887b02910)

### Other

- Bump version to 0.3.7
 ([d5467b1](d5467b1e3767412a22c14e7f6587ad5887b02910))
- Improve pdf output
 ([aeed782](aeed78214a0fafa77e825e5fea119f29677fb9dd))
- Change ai command output formating

Adding comma separated decimal and widen the columns as SHIB fits as
it there is typically "millions".
 ([0dafd47](0dafd4783e589145483d40044bf12e820b720e65))

## [0.3.6] - 2022-02-03

[3aa6c3e](3aa6c3e4685daa35a0a779b0135efc3ea12e57fa)...[6c739e1](6c739e13f5e49111fff92ed680567c1b17911ed7)

### Other

- Bump version to 0.3.6
 ([6c739e1](6c739e13f5e49111fff92ed680567c1b17911ed7))
- Allow time parameter to get_account_info

I'd added this time parameter thinking I could get historical data. Nope
it's related to the receive window.
 ([3dc03e9](3dc03e91160b114ff799742e1ee5c66492576b7a))
- Use get_kline_of_primary_asset_for_value_asset

Allow account info to report usd values for additional assets such as
SHIB.
 ([20cec7c](20cec7cfaa1f1d9199a41e83b6859260e20ce991))
- Tracking some fees in Distribution, withdrawal and deposits

 - Subtract Distribution and Withdrawal fees directly from the associated
   asset record.
 - For USD Deposit tracking total_usd_deposit_fee and
   total_usd_deposit_fee_count as they are tax deductible but.
 - For Crypto Deposit Tracking total_crypto_deposit_fee_count is counted
   to see if they need to be accounted for on a per asset collection.
 - Track Crypto Withdrawal fees
 - Added a TODO for usd_cost_basis
 - Added adding missing quote_asset or fee_asset and issue a WARNING.
 - Cleanup debug
 ([303225c](303225c5d880584a0f8964d25cabf9dcca355fed))
- Use trade_asset

At the moment I'm off a little on the ETH trades.
 ([fe0ad33](fe0ad33aabc61a5e766f56366e878862d1749225))
- Need to work around mutating multiple entries in AssetRecMap

Mutating multiple entries of a collection by using references to them
violates the single mutator guarantee in Rust. Fix for now is to add
associated functions to AssetRecMap which gets a mutable reference,
mutates a field and then drops the reference. Thus only one reference
is held at a time.
 ([a08ad79](a08ad79d363d03c7676a684d57a4eef41a4c7c53))
- Remove some of the debug
 ([f78b1fa](f78b1fa8c5c25dea11d39fb573c2da517a8e451e))
- Change dbg_x to accept RectMut<AcessRec>
 ([bac9589](bac95892c2494c203d1e38461ace927eb6186f05))
- Change entry to mut entry and remove borrow_mut()
 ([9777da0](9777da0b738861b4abc24e9b3df8303e66546b18))
- Add trade_asset

The trade_asset function needs to change multiple records in a single
call. To accomplish this I'm using RefCell which allows this.
 ([3236dda](3236dda717594c2c169ba94b37a82a564176268e))
- Add obid

Displays orders by numeric id
 ([2497923](2497923b51e66abb1c034a2fa7a129e5f2e0ef6a))
- Use asset_value and asset_value_usd consistently

Also add dbg_x to find where BTC was going negative.
 ([eba1f11](eba1f11eab46e1312ebafc05523e7b9585500472))
- Change processing


Add udf command, "update_distribution_file". It takes an input file and
checks each transaction updating those without a USD value. And then
writes the results to the provided output file.

Now pdf command only takes the input file and assumes the file is good
and then processes each transaction and tracks the "current" value of
each asset and then at the end outputs a report of the final end of year
value for each asset.
 ([51a0508](51a05084c7f1acbdcc05735b2cb1a924c687370e))

## [0.3.5] - 2022-01-30

[b2b9ee1](b2b9ee1eeb88600a92316a50650a9b1a490729e8)...[3aa6c3e](3aa6c3e4685daa35a0a779b0135efc3ea12e57fa)

### Other

- Bump version to 0.3.5
 ([3aa6c3e](3aa6c3e4685daa35a0a779b0135efc3ea12e57fa))
- Add support for Deposit category
 ([9541d26](9541d2682c3748a4961a157e9cb798138176ab89))
- Add update_all_usd_values

Separating the updating from processing simplifies the code:

 - All *_usd_values are updated in one place
 -- Previously I wasn't updating the fee or quote
 - update_all_usd_values doesn't need the data parameter.

The process_entry is simplified:

 - It doesn't need to be async
 - dr parameters is immutable
 - unwrap() can be used as fixup guarantees the value is not None
 ([dfdfa13](dfdfa133bc8b9f971cbd65e21fd3f840e70a9023))
- Fix a clippy error on nightly
 ([e5c2b4f](e5c2b4fe20ac42b29ad09d3a3fe1a1d37975e917))
- Initial processing of every transaction

The code now porocess all transactions in all cateogories and operations
that are in binance.us-distribution-2021.csv.

By processing I mean each transaction is used and any missing USD fields
are updated using the closing value of the nearest minute kline.
There is a number of asserts in the code which validates that expected
counts are appropriate. In particular all "unknown" counts are zero which
means all transactions were processed.
 ([59c3952](59c395232e755f2b5f115037e66ca31970861dcd))

## [0.3.4] - 2022-01-27

[04a2b2d](04a2b2d1bbdda5004edabd50e9f227f8e8f6beec)...[b2b9ee1](b2b9ee1eeb88600a92316a50650a9b1a490729e8)

### Other

- Bump version to 3.4
 ([b2b9ee1](b2b9ee1eeb88600a92316a50650a9b1a490729e8))
- Improve error handling of File::open & File::create
 ([457e355](457e35586f818df8b37e6a0325bb0b5c89473bd4))
- Update clap to v 3.0.13

Need to add clap features "env" and added a few simple test_params.
 ([bf6a245](bf6a2457945e882cecab8354919c1b7fccc5c4c0))

## [0.3.3] - 2022-01-26

[54506d4](54506d4c7c8fd11b3206bcccf8029f1bbde7dfb1)...[04a2b2d](04a2b2d1bbdda5004edabd50e9f227f8e8f6beec)

### Other

- Bump version to 0.3.3

Also, only display the "Records with no USD value" if there were none.
 ([04a2b2d](04a2b2d1bbdda5004edabd50e9f227f8e8f6beec))
- Small refactor of main.rs
 ([763e3e3](763e3e3cd6f24d176a0406f5b8e7f1fca08f2759))
- Update dependencies

Plus some necessary tweaks to get `cargo test` working again.

I used `cargo update` and then manually updated Cargo.toml from the
versions in Cargo.lock.
 ([b0e62b2](b0e62b2e63aca9fb58c2f0c3f0a70d4dd549e3bd))
- Comment out printing config and subcmd for pdf command.
 ([1f7271f](1f7271f69bf334b5f80be912c62e60b5779ac666))
- Use BTreeMap instead of a HashMap

The performance was the same and now the output of assets are always
sorted. Also renamed ProcessedData.hm to .arm.
 ([fdd0532](fdd0532692c80309885af391447f386f364cc57d))
- Rename process_hm_entry to process_entry
 ([04931ec](04931ec147be8c86ca2778c8445ae724446c98d6))
- Add empty_rpa_usd_value

And append _count to empty_rpa and empty_rpa_usd and also output
empty_rpa_usd_value so we can see how much the empty data was.

Answer: #636.98

The other insteresting fact was they were all in December starting on
2021-12-02. And it turns out from 2021-12-30 they were all for NANO.
On the last day of the year only the USD transactions had USD amounts, it
looks like all the others were had empty USD values.
 ([c21b6c4](c21b6c47188098b2e4354c7a68ff0ff7f189b3af))
- Remove noise added to get_klines
 ([1b0af24](1b0af24df3df094ad07caff5e4bcfb5b669bc9ca))
- Add START-TIME-UTC optional parameter to skr
 ([66751b8](66751b8687522e03b25e7eaee3bdd537da006101))
- Calculate rpa_usd
 ([db1ba74](db1ba7477747ba912038b28381b9a69bc9ae8191))
- Add get_kline_of_base_asset_for_value_asset
 ([cd83b65](cd83b6504d5fcf13c7973c7014f23a6154a1f5c2))
- Change process_hm_entry to async

And get_line for entries without USD values. But ATM I'm not calculating
the actual USD value but it did succeed in processing all of those
records.
 ([b6a4cc9](b6a4cc973f760ccf9b181363fb712e729bc41c3d))
- Do not use iterators

I've removed the iterators because I couldn't get passing process_line
as an async function working and searching didn't yield great hits:
  https://www.google.com/search?q=rust%3A+how+to+pass+async+function+pointers

The next commit will make process_hm_entry async and it will "just work".
 ([500f254](500f254235ffc78c5d5700480d02226a69a52fef))
- Extract process_hm_entry to global scope

This is so it can't accidentally "capture" anything and maybe it
 ([25976cb](25976cbd871065d6b445864b46b696a52452154f))
- Add type ProcessLine

Initially I tried:

 type ProcessLine = impl FnMut(
    &Configuration,
    &mut ProcessedData,
    &mut DistRec,
    usize,
 ) -> Result<(), Box<dyn std::error::Error>>;

But the compiler didn't like it and complained that `impl Trait` was
unstable. So I removed it:

 type ProcessLine = impl FnMut(
    &Configuration,
    &mut ProcessedData,
    &mut DistRec,
    usize,
 ) -> Result<(), Box<dyn std::error::Error>>;

Then it said try dyn:

 type ProcessLine = dyn FnMut(
    &Configuration,
    &mut ProcessedData,
    &mut DistRec,
    usize,
 ) -> Result<(), Box<dyn std::error::Error>>;

And then it complained Doesn't have a size known at compile-time. So then
I tried just a `fn` and that worked:

 type ProcessLine = dyn FnMut(
    &Configuration,
    &mut ProcessedData,
    &mut DistRec,
    usize,
 ) -> Result<(), Box<dyn std::error::Error>>;
 ([65a0cb0](65a0cb0e6349b167cf0a9d57aad7df51cac234be))
- Prepare for process_line to be async

The process_line function can't be a closure as closures cannot be async
on stable. So Create struct ProcessedData and pass it along with
Configuration as parameters to process_line.
 ([38050ba](38050baf403b2c0b2f784bdd7c2060f47689928a))
- Update README.md
 ([5c852fb](5c852fb71f9c84fbe47a9c615f27d17d39a7a080))
- Change pdf to accept two parameters

The first parameter is the input file and is required, the second is
optional and is the output file. In future changes the second file will
be the same as the input file but also has fields that may have been
updated. Such as the missing USD amounts in some Distribution records.
 ([f94a15e](f94a15e1ca501aa1b7bfa1e1b3eef794b523dc0c))
- Adding headers: HeaderMap to ResponseErrorMap
 ([163556f](163556f27651b9761325869da43cbf5146e738af))
- Add time_ms_to_utc_string

And rename se_time_ms_to_rfc3339 to se_time_ms_to_utc_string and have
se_time_ms_to_utc_string use time_ms_to_utc_string and fix tests.
 ([6a0ce2b](6a0ce2bfc5b1b6b27ef66d49d08bcb2f4592b7b9))
- Fix bug in some of the args.

I believe I mistakenly confused argument index and display_order.
 ([83e3849](83e38496e6ab91485ac3047f85b68faa556f0487))
- Fix typo of asset
 ([881efc3](881efc33279b7298a2475cb3ebc812a16ca1c27f))
- Change part of the test_convert to BNB to BUSB

As it sometimes fails when converting to USD:

  ---- binance_trade::test::test_convert stdout ----
  thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: "convert error, asset: BNB not convertable to USD"', src/binance_trade.rs:396:14
 ([6bcab92](6bcab92078462ad29cd8b2cb03fc56c46bfc895b))
- Add se_time_ms_to_rfc3339

Serialize time_ms to rfc3339.

Also renamed mod tests in de_string_to_utc_time_ms.rs to test.
 ([803c05e](803c05efa611ec96090610900aa278944b8f3445))
- Rename de_string_to_utc_time_ms and AddTzUtc

To de_string_to_utc_time_ms_condaddtzutc and CondAddTzUtc respectively.
And then use de_string_to_utc_time_ms_condaddtzutc to deserialize DistRec.

Note: This slows down the processing of binance.us-distribution-2021 from
5s to 7.8s with --no-verbose
 ([05aea8d](05aea8dda61a8a610c8d1b9328d968f59a76ce72))
- Rename total to total_rpa_used
 ([5a9f7eb](5a9f7eb3aad1ae7f9ac7adf0491fe787ca51f843))
- Add FILE argument for pdf
 ([e2a81e5](e2a81e52865a06a8a7a8f02c7477e7632c613f6a))
- Minimal processing of all categories

- Except for Distribution just count the other categories.
- Use config.verbose to manage the progress output. Turns out the
  current progress output slows the processing down by > 3x. Without
  it it takes about 5s to run binance.us-distribution-2021.csv and about
  18s when verbose is enabled.
- Output counts for the categories.
 ([73dedd2](73dedd2704db39713c2084bb8985c4b79919a885))
- Support massaging the timezone information

common.rs:
 What I wanted was that if a user does not provide a timezone than assume
 the users local timezone. That wasn't working so added TzMassaging.

binance_get_klines_cmd.rs:
 Changed get_klines_cmd to first assume LocatTz and if that fails then
 try HasTz.

arg_matches.rs:
 Added user help for the possible formats for --start_time.
 ([c9c9023](c9c9023bb9f2178f4e06e6f17ad6ae0ed3cf1ba5))
- Fix dt_str_xxx_to_utc_time_ms working

Previously, it was returning a UTC value that was actually off by the
local timezone. It now returns the proper value. Spectifically time_ms
is 0 for 1970-01-01T00:00:00.

Also some cleanup.
 ([8fd593a](8fd593abd1d844b034816c5a19ffacdcd2704616))
- Rename de_string_to_time_ms.rs to de_string_to_utc_time_ms.rs
 ([fbaef81](fbaef81330e83d1f1a3bd81c03d47ff3f238ad94))
- Fix tarpaulin config file

Needed to add [test_config] to remove an error which I hadn't seen because
I haven't been running it manually recently and there must have been a
change.

Changed verbose to false.
 ([57b0d24](57b0d24a55661a14525037fca9bd1339b2a263ae))
- Fix poor code found by clippy
 ([b5fb60a](b5fb60ac83dbcd5f9979a21f451c1a35a564d7c0))
- Add de_string_to_time_ms

This allows using serde for time strings to time_ms as i64.

Also added dt_str_space_sep_to_utc_time_ms and
dt_str_tee_sep_to_utc_time_ms which de_str_to_utc_time_ms uses to handle
both T or Space separators between Date and Time. In addition subsecond
times are supported using the '%.f' format spec.

Changed error message to say "naive" rather than "local".

Finally added some tests.
 ([ebc12de](ebc12dead9988a678079ed60f23dce186ae5ee1b))
- Processes only Category == Distributions

 - Added some Module documentation with some information.
 - Renamed init_hm_entry to process_hm_entry
 - Added empty_rpa and empty_rpa_usd to see what fields are missing.
   Turns out empty_rpa is always zero so added an assert but there are
   quite a few, 6,000+ empty_rpa_usd so will need to add code to lookup
   and calculate Realized_Amount_For_Quote_Asset_In_USD_Value.
 ([5ffe907](5ffe90757a4bc8efb5d1d796ad961e567b721314))

## [0.3.2] - 2022-01-14

[5d0129d](5d0129d085004e6d4d0d931ef5782d99fea724f8)...[54506d4](54506d4c7c8fd11b3206bcccf8029f1bbde7dfb1)

### Other

- Able to process file ./data/binance.us-distribution-2021.csv

This is probably INCORRECT, but does produce some numbers.
 ([54506d4](54506d4c7c8fd11b3206bcccf8029f1bbde7dfb1))
- Can read a binance.us distribution file into DistRecs

Right now I just display some of the data.
 ([4ae9ea5](4ae9ea5826f4a8eaa72c4bb34bd6c1258fee4a43))
- Add binance_distribution_processing

Also known as the "pdf" command. Currently does nothing!
 ([cb14d5e](cb14d5ea1f5dd70ec3747df8c852c7fbb29e9f15))
- Add process_order_log

Initial commit just have it display TradeResponse::SuccessFull and
SuccessWithdraw
 ([a9ff41f](a9ff41fe60dacc9b08e80c16e7a5ee658b372025))

## [0.3.1] - 2021-12-27

[5cc3882](5cc3882c7dea6e62ffe8e2b954614ca3c60f5980)...[5d0129d](5d0129d085004e6d4d0d931ef5782d99fea724f8)

### Other

- Update to version 0.3.1
 ([5d0129d](5d0129d085004e6d4d0d931ef5782d99fea724f8))
- Add serd(default) for new files in WithdrawParams
 ([3ff5c88](3ff5c884fe97ca749ad5812f63ab67d4b7bbcc73))
- Add --verbose and --no-verbose flags

These are used to control the "Updating" message in
AccountInfo::update_values_in_usd. It's available in config and
likely will be used for other output in the future.
 ([f79afca](f79afca4a79733b7c5930c9702865a541dcdeb80))
- Add --keep-min to withdraw sub-command

This allows the user to specify a minimum amount of the asset to
NOT withdraw and instead keep in binacne.us.

Also allow the AMOUNT and --keep-min values be in percent, dollars
or quantiity. And pring the value withdrawn in USD.

NOTE: When dollars are used by prefixing with Dollar Sign, $. The value
must must escaped in single quotes or being preceeded with a backslash:

  '$456' or \$456

See binance-cli withdraw -h for more examples.
 ([dbb0627](dbb0627a05cf074ea044a5f8711ed5ba81c96c65))
- Fix bug where quantity was a multiple of lot_size

This caused a withdraw to fail, by adding a call to
adj_quantity_verify_lot_size this will not happen.

Also, added saving the org_quantity in the params so if it were to happen
in the future we'd know the original quanity.
 ([c9e3ae6](c9e3ae65eae40ba0498a000c9fb845302b4afd68))
- Cleanup imports

Remove the #[allow(unused)] and then remove unused imports.
 ([7a6cc5f](7a6cc5f1697ebb2c230393ec743fbadcd93064fa))
- Add confirmation_required

There is now confirmation-required and no-confirmation-required command
line flags as well as confirmation_required Configuration option.

This allows the program to not require confirmation when using the
auto-buy and auto-sell sub-commands. BE VERY CAREFUL!
 ([ded6639](ded663932b36865889fe2706a0b132b956fcb190))

## [0.3.0] - 2021-11-08

[3dfae08](3dfae0832859194068c1cce88281070d5a705630)...[5cc3882](5cc3882c7dea6e62ffe8e2b954614ca3c60f5980)

### Other

- Update to Edition 2021 and bump version to 0.3.0

Also update to the latest versions of the dependencies. I used
`cargo update`, `cargo upgrades`, `cargo upgrade` and `cargo outdated`.
Outdated did the "best" but it didn't update [dev-dependencies] and
[build-dependencies] to the very latest so I manually updated them.

The other goal of the updating to the latest versions was to eliminate
the duplicate versions of the same crate. Using `cargo tree -d` shows
what it thinks are duplicates but all of them are actually the same
version so I'm not sure what it's showing.

Add small tweaks to binance_signature and have it use new_from_slice
as new_varkey has been removed.

Finally, edition 2021 doesn't work on the old nightly and rustfmt is
now working on nightly. So toolchain is changed back to "nightly"
instead of "nightly-2021-03-25". Also, clippy complained about
additional items so fixed those in binance_auto_buy.rs,
binance_auto_sell.rs, common.rs and configuration.rs.
 ([5cc3882](5cc3882c7dea6e62ffe8e2b954614ca3c60f5980))
- * Fix warn semicolon_in_experssions_from_macros and lint errors

  wink@3900x:~/prgs/rust/myrepos/binance-cli (Improve-the-my-trades-command-mt)
  $ cargo build
     Compiling binance-cli v0.2.0 (/home/wink/prgs/rust/myrepos/binance-cli)
  warning: trailing semicolon in macro used in expression position
   --> src/common.rs:95:70
    |
 95 |         InternalErrorRec::new($c, std::file!(), "", std::line!(), $m);
    |                                                                      ^
    |
   ::: src/binance_auto_buy.rs:54:20
    |
 54 |         return Err(ier_new!(8, "Missing `buy` field in configuration").into());
    |                    --------------------------------------------------- in this macro invocation
    |
    = note: `#[warn(semicolon_in_expressions_from_macros)]` on by default
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #7981

* Fix lint errors

  cargo clippy was failing, fix the errors.
 ([21713b2](21713b23fa25927e2b0bbf8dd0d59c2fad045ec6))
- Improve the my-trades command "mt"

Ouput more granular data and use trace! to see more information when
desired.
 ([860f675](860f67540c4eb3d8398e2fac073e261d61169276))
- Fix enum RateLimitType

This fixes the error Kendall reported:

 Kendall Saville, [30.06.21 09:40]
 Running `target/debug/binance-cli auto-sell -t`
 Error: Error("unknown variant `RAW_REQUESTS`, expected one of `RAW_REQUEST`, `REQUEST_WEIGHT`, `ORDERS`", line: 1, column: 325)

AFAICT today is the first time rateLimitType has been returned in
exchange info and the RAW_REQUEST is really RAW_REQUESTS (ends in 's').
 ([7d36938](7d36938e8b54495e71f03cba1256119a3d833adb))
- Fix some clippy lint errors.
 ([32d53be](32d53beb94547370668c7d7c8bb27da86289e790))
- Add additional information to some errors.

Especially the symbol name.
 ([fc85ca3](fc85ca3e5b6c3a13a93a3cce0b067bf7b1eea67a))
- Fix bug not selling ETH

I was doing rounding which would round values up sometimes and if it
rounded up to a value greater than the balance.free it failed. So
removed the rounding when selling and withdrawing.
 ([27c0ca5](27c0ca546a7e57c1b6c5a557cda4cc85c98bafb4))
- Add time_ms_utc_to_naive_local

At the moment for displaying timestamps for some debugging I did,
but not used yet as no tests.
 ([9ea7ac5](9ea7ac5e4cb4f1928da29bd7781e61e547bc9c0a))
- Improve Keys associated methods

The previous technique for get_xk_as_bytes_or_err didn't work and instead
I used get_sk_or_err() which returned a String which was a clone of
Keys.secret_key and then I used String::as_bytes() to convert it to a
&[u8]. This works but wasn't "obvious".

The new code changes get_xk_or_err() to return a slice, &str, and
get_xk_vec_u8_or_err() returns a Vec<u8>. The advantage is that now
get_xk_or_err() doesn't do a clone() but although there is a clone
with the String::into_bytes() it feels more ergonomic, IMHO.
 ([56e9e65](56e9e6581e67b422b6a995b69d317d13a5940a22))
- Cargo update and fix a clippy lint.

I hope!
 ([97e22af](97e22af0a776183ef281ee656a476ead120e2be6))
- Add response_body to WithdrawResponseRec

This allows us to see what the actual response was for diagnostic
purposes. Especially in the case of additional fields are added in
the future.
 ([43c381a](43c381af55ddf8868cbda1a6d90ec402f7044a63))
- Fix typo
 ([978eb48](978eb4853c6e8d56f8158057b765ca7dde419d47))
- Simplify InternalErrorRec use

Since it now implements the Error trait it's no longer necessary to do
a to_string().
 ([f10a4d5](f10a4d5b10467f33a953bdee8b692f7ab703a72a))
- Make Keys.api_key and secret_key Optional

I want to do this because I don't want to require a config for all
subcommands. Thus I had to change handling ErrorKind::NotFound from being
an Err to using the default. That meant we ended up with an error message
from the website rather than catching it earlier.

Also, I changed InternalErrorRec to implement Error so I can just use
ier_new! directly in get_ak_or_err and get_sk_or_err without having to
create convert it to a string.

In addition I have two fix a couple tests in binance_trade.rs to set
some default keys.
 ([8776475](87764755a7ecf02354bccf8aa896bfe951932dea))
- Tweak WiddrawResponseRec

Have msg and id default to empty if not available.
 ([c78eaec](c78eaec868219b8790da254fccbfd735f5b6f8a1))
- Add trace to display response status and body
 ([96c8e28](96c8e281d1f96636d308afdde9b12d07ee506eb9))
- Report Error file not found

If config file is missing.
 ([bcbe5ce](bcbe5cea215fa207d562f0da208290fbeeb541a3))
- Clean up logging and output
 ([e7b00ed](e7b00edda833fb7543f7d1a37ca57d5ea33061d3))
- Fix getting secondary_address and label and reorg
 ([ca688a3](ca688a3b2fa4b1584a596b74baef97a34a5ca637))
- Fleshing out
 ([7ddf538](7ddf538b707bc593ff136a58023843abd97f6e5d))
- Adding withdraw WIP
 ([3215053](32150531f4ab9a3e7ffe684622ac581ea3c46a22))
- Rename verify_quanity_is_greator_than_free

To verify_quanity_is_less_than_or_dq_free. Also, fix an upwrap handling
of order_log_path.
 ([cc1ceca](cc1cecaec1d747ee94501abb5b975565d6ecc969))
- Update README.md

Add instructions to clone repo.
 ([6b70a2a](6b70a2ae955bd28344b87b0630f35381d5ba9e43))
- Fix bug, skip if symbol if not found while auto-selling

My test was to stringent, when a symbol, such as BCHAUSD, is not found
while trying to decide what values to sell just ignore it. In this
particular case it was "suspended" and now it can't be sold.
 ([17ae8e2](17ae8e2461ec26f82be4f35eea3b77378035f168))
- Update README.md and config.toml

Add additional information, clean up language and fix typos :)
 ([2ad3160](2ad3160a2452e558b18fb1291afe1006317bbae8))
- Fix bug in auto-buy

 - The logic required a buy record to have a quote_asset.
 - Added a SKIPPING message on bad symbols or not owned.
 - Also fix a typo in a trace! statement.
 ([0548006](05480062c57f531176bc47c964f3268e412a35cc))
- Fix typo of order_log_path in cconfig.toml
 ([3821871](3821871d3b70bc3fc092af2dd2013103b8bd4678))
- Add fcdh and fcwh subcommands

These display the fiat currency deposit and withdraw histories for an
account.
 ([9a950c8](9a950c8b1d9a2b9de66a66d2b2455ba6e84f7321))
- Fix test failure

If there is no config file, such as configs/config.toml is missing, we
should not fail but just use Configuration::default().
 ([980af94](980af94a9174498f2a86dd408cf3cb36cec597a7))

## [0.2.0] - 2021-06-02

[a8c726d](a8c726d4734b7b1e42aa5ed5d9f68600f3eab970)...[3dfae08](3dfae0832859194068c1cce88281070d5a705630)

### Other

- Update README.md and bump version to 0.2.0
 ([3dfae08](3dfae0832859194068c1cce88281070d5a705630))
- Change Usage so ver is app-ver
 ([68783fd](68783fd886bcd12bbe6a500635fbc9f3742086fd))
- Add more documentation and errors if used

Also illegal subfields in buy and keep so that this config.toml
can NOT be used accidentally.
 ([a6cd35a](a6cd35af145fd3a86eac4b15f37938c267b3b45c))
- Change Configuration buy and keep to be optional

This decreases the possibility that auto-sell will sell everything, now
you must have a keep field that is empty.
 ([99d1100](99d1100ed980e3e42c32e5c681cce5a41c8a6764))
- Cleanup InternalErrorRec Display implementation

 - ver is now called app-ver
 - app-ver is now displayed without escaped double quotes
 ([2393725](239372555bd5ee662e41d534277e8ee4510868e1))
- Default to configs/config.toml

This means we will never accidentally execute the example config.toml
in the root.
 ([1d590b1](1d590b194e4d7e8c406c30dfd1e5b0c58960c9e9))
- Refactor binance_history

This is better, we now have DepositRec and WithdrawRec.
 ([a5ddc86](a5ddc8638f7755401208789b50a063439903aa96))
- Move api_key and secret_key into struct Keys

I then implement fmt::Debug for Keys which does display the first 6
characters of api_key and displays the secret_key as 6 asterisks.
This makes it "impossible" to accidentally output the the keys,
which I was doing when Debug printing config. Such as:

 trace!("{:?}", config);
 println!("{:?}", config);
 println!("{:#?}", config);

This cannot happen now!
 ([7edf47a](7edf47a6d3c05a53085471e46e80d57394abf5e9))
- Ignore configs directory
 ([b3038a6](b3038a60715f5d26a72d72ea67c4bb8b829924c7))
- Update config.toml

Add comments explaining keep and buy
 ([bc28838](bc288382e89824666d72faa6e6c019e57b4d4a4e))
- Added binance_history

Which displays the deposit and withdraw history of the account based
off binance_my_trades. I started down the path were I thought
HistoryRec was going to be the same for deposit and withdraw so
used one struct. But they are only "mostly" the same so this needs
to be refactored.

Added "dh", "wh" and "mt" to be displayed at towards the top of the help
and "ao" above "ol".

Modified mt subcommand to use the histories to calculate the
equivalent of Total Coins in from the ai subcommand. On my account
its pretty close for ETH. The ai subcommand reports 7.43502526 and
mt subcommand reports a little more, 7.43688064. This is a difference
of 0.0018554. My current guess is there are some transaction fees that
are not being accounted for.
 ([98bec9d](98bec9d21ab976ddd97682c7bf39494e7355ea0f))
- Cleanup binance_my_trades error handling
 ([bbfc55a](bbfc55a62e0e24c76e5aedbb8e23b0fec61802d0))
- Change my trades total calculations

When buying the total_qty and total_quote_value increase and
they decrease when selling.

Added TradeRec.is_buyer_factor which returns dec!(1) if buyer
and dec!(-1) if seller.

Added some tests, yeah me!!!
 ([03b30e1](03b30e1488484c0a97282f9c5cc18c67b5713882))
- Add dec_to_separated_string

Also add a couple tests.
 ([b2a3aeb](b2a3aeb4799ee82ab7d770a95773e97e3997ecd3))
- Change subcommand st to mt

A "st" is for symbol trades but it actually uses the binance
myTrades api so "mt" is more descriptive as it actually
is showing "my trades" not anyone elses trades.
 ([5632ac4](5632ac4b3b0bd46a5d427fba3f7c7df966c33abd))
- Itgnore directories /data*/ in git
 ([6b9e04f](6b9e04f41c0e644e593dd92c244ce219a660417c))
- Fix bugs in index parameter

When adding display_order a search and replace I did changed all
"1)" to "10)" this was a mistake and caused clap to think that
the first parameter needed to be the 10th.
 ([4b3f940](4b3f94028432edb0d525bd53aac39eb483c02b9d))
- Format money better

Added common::dec_to_money_string which uses rusty-money crate.
 ([5a188f6](5a188f67f9727f1a79a7df266b2464f5137b29a5))
- Add display_order to subcommands

This aranges them in a better order.
 ([cfab480](cfab480ac824700211acf7ce81ea06eaefe45779))
- Rename package to binance-cli
 ([5203c42](5203c423638c9da9be2c1a422fd3d062330b0b41))
- Fix formatting
 ([312e2f8](312e2f8a61f3b5ba432e507e8394385c43d1cb1e))
- Get auto-buy working

buy_value wasn't calculated correctly (wasn't dividing by 100) and
I got a quote precision error so I needed to round_de(sym.quote_precision).
But the quote_precision needs to be a u32 not a u64. So I had to add
the de_string_or_number_to_i32 and u32 and change the various _precision
value in struct Symbol. Fixed the output.

Also, added checking for asset != quote_asset so we ignore "USDUSD" and
the like. Not sure why I didn't run into that before. Added rounding
sell_qty to quote_precision in auto-sell as I did with buy_value. And
some cleanup of the output.
quote_precision in the
 ([81ef0ab](81ef0abeffbdd3e36ed54dfa127e2638a4647410))
- Add no-test flag

Enables test mode, i.e. the opposte of --test and -t. Note also
there is no short name as I regard this as "dangerous".
 ([fdaac77](fdaac77684f91325edb7c4bd2ce1f24f37478064))
- Update launch.json

 - Add do-nothing, auto-buy and auto-sell
 - Change the "name" field to "Debug xxxx"
 ([16f01f5](16f01f5f9dde474ffd2da4128e0f33b658d2adfe))
- Default test = true in config.toml

Seems safer
 ([88d472d](88d472d067e0922054fa4b27b552b5f465527d1f))
- Update README.md and config.toml

Add .vscode/launch.json
 ([0639db9](0639db9c418848d4a9807400fa4ec8c32195c8cf))
- Initial testing of sell|buy-market-value

 - Fix bug in sell_market_order_cmd, SELL not BUY!
 - Fix typo of Value should have been VALUE in arg_matches()
 - Add printing version when printing Usage.
 - Change Configuration::new to return errors if processing
   or reading the file has problems.
 - Change log-path to order-log-path
 - Remove function_name and named. It has runtime cost and
   rust-analyzer doesn't like it.
 ([e865e61](e865e612c2050f721416f122da804274aeeb3f60))
- Clean up main
 ([cfe9f3d](cfe9f3d31444dff961b8c8678c4c898e2964637b))
- Added tests to configuration

Moved tests to configuration from binance_auto_buy
and binance_auto_sell.

Added some additional tests and defaults.
 ([ee57c10](ee57c10aa927389936a00ebdbc3f17ba866c9dbd))
- Add auto-buy, buy-market-value and sell-market-value
 ([0244a83](0244a831dd2ba87b482481eff4b827faa765c836))
- Add the short git sha to version

Using vergen and a build.rs to be able to construct the
desired version. And lazily create static VERSION.
 ([85c8b71](85c8b714969848f78f514cb6e72f7ad5ae2c8b7f))
- Update README.md
 ([cabd90c](cabd90c114dd60534f408bf5c964358851545299))
- Fix cargo-precommit

Needed an EOL on last line so `cat cargo-precommit` works :)
 ([f591eae](f591eae152ca63b68fb45e2f31a4f2f6a0d90e00))
- Clean up

 - Use a match statement rather than if else if series
 - Determined that by setting global(true) for Args they are
   allowed to be used after the subcommand
 - Determined that Args can be declared outside in a let statement
   and then "cloned" in an App. Thus reducing duplication. At the
   moment I'm not using that, but could be useful in the future.
 ([d3cde28](d3cde2824def47176eb0e467bbb08a19912c0c92))
- Add subcommands

Compiles and runs but not happy, especially the if else if for issuing
the subcommands. Also I don't like that some options/flags are passed
before the subcommand and some afterwards.
 ([fdf69eb](fdf69eb97e3cd509bcb1597d240e5a7d64480035))
- Remove BinanceContext
 ([4180f60](4180f60f9f94e3db1bef9148fbea175ab378379f))
- New configuration and argument handling
 ([8304e5e](8304e5eba06fc8dd844f0d869136f94e1e0d54b3))
- Fix SKIPPING message on internal errors

To much information and are expected
 ([e346aa3](e346aa3504ec5bea2662a3d61ee1e07273f5fde7))
- Fix typo
 ([83e7eb0](83e7eb0ab233f6029300aa11daf59dcccc16271e))
- Moved Configuration to common

Originally ti was ConfigAutoSell but with this change not only auto-sell
but buy-market and sell-market require the configuration file which
is used to get the KEYS.

Also changed fmt::Display for Full and Result TradeResponseRec and
cleaned up how auto-sell output.
 ([b593585](b593585b5e197b291d01263a453102413791cb27))
- Change logging.

Have market order open the log file so it can log errors it finds
when it verifying parameters.

This necessitated the following changes additional changes:

- Make order_log_file and log_order_response to be public
  and clean up order_log_file
- Have log_order_response take a mut dyn Write rather than being a
  generic function taking a Write. This allows it to be used outside
  of binance_trade
- Have binance_new_order_or_test take the log_writer as a parameter
  rather than opening it locally.
 ([723017a](723017a80344f7c83b9d5e118d0039202a502ea1))
- Clean up

I still need to make use of InternalErrorRec via FailureInternal so
they can be output to the order_log. Also, it will provide more
precise location information for the errors as well as useful for
println! and trace! of errors.
 ([cb77d31](cb77d3157b41d581d948800bd2f878a78e97bfee))
- Change auto_sell to skip errors

Added order_log_path to ConfigAutoSell.

Previously in auto_sell() if market_order() returned an error
it returned immediately. Now Skipping is printed.

Also, changed adj__quantity_verify_lot_size() to return quantity and
never return an error. And then changed the call to it in
market_order() appropriately.
 ([e7dcbb2](e7dcbb2b21a185d7ce131985334c05461ce623bf))
- Improve output

 - Added verbose parameter when updating account info
 - Output all "SELLING" assets after those we are "Keeping".
 ([e89b894](e89b8942b5549ee2764f2c2755ac4a448dd0b41e))
- Change test mode on auto-sell

Do not ask "Are you sure, type Yes: " if test mode "-t" flag is enabled.
 ([5cce707](5cce707f840721db3aa4cee8651fdaa4990c8908))
- Return TradeResponse from market_order

And improve the output of auto_sell()
 ([3191e59](3191e596e72ad3ec3c2c736da69104b74a716612))
- Implement Display for Side and TradeResponse

Also add PartialEq and a trace! for Side. And in FullTradeResponseRec
changed side to be type Side instead of string.
 ([43f0bd4](43f0bd4d4d6ceea8876badca88c7ca78be90f26c))
- Improve auto_sell

Added an are you sure option
 ([edd7663](edd766329425dd4d2c86c724c0c2c121df929dd4))
- Add auto_sell_cmd

And actually calling market_order and added test -t parameter to
auto_sell_cmd which is passed to market_order.

binance_account_info:
 - Changed update-values to update_values_in_usd
 - Changed Balance.value to Balance.value_in_usd

binance_auto_sell:
 - Added auto_sell_cmd and AutoSellCmdRec
 - auto_sell gets new parameters, ei and test
 - KeepRec add sell_to_asset, renane xxx_value to xxx_value_in_usd
 - Use market_order
 - Added some trace! calls

binance_context:
 - Added AutoSell to SubCommands

binance_market_order_cmd
 - Changed println! to trace!

main:
 - Added calling auto_sell_cmd
 ([7e6c8c3](7e6c8c3af07cecc5cf3dc8872b9561829afb7a6e))
- Fix bugs

binance_auto_sell
 - incorrectly calculated keep_qty, it would be incorrect
   if keep_qty > owned_qty.
 - Added rounding when displaying balance.value,

binance_account_info
 - Incorrectly calcuated balance.value, needed parens
   around sum of free and locked.
 - use get_kline intead of get_avg_price.
 ([e6300d2](e6300d2ea730c4e0a637615c57069c2fcfb07089))
- Change binance_trade::convert to use get_kline

I do this because get_kline uses in interval of 1 minute, where as
get_avg_price uses an average over 5 minute value.
 ([c04d8bb](c04d8bbfb0a434f5dfbb30e6f488971b4a9f3657))
- Rneame binance_market to binance_market_order_cmd

 - Rename buy_market_order to buy_market_order_cmd
 - Rename sell_market_order to sell_market_order_cmd
 ([cea57a6](cea57a6ad3abac55379913e420a16e98c051411d))
- Add BuyMarket and SellMarket subcommands

 - binance_order_response added SuccessTest and
   TestTradeResponseRect so trade response aren't SuccessUnknown.
 - binance_new_trade_or_test report SuccessTest
 ([d078071](d07807193c190a2f5623990732a8463583360898))
- Add klines as a subcommand

This makes the command line parameter handling better.

 - Added binance_get_klines_cmd with get_klines_cmd
 - Have get_klines conditionally send startTime instead of get_kline
   and simpliy get_kline.
 - Added to common naive_to_utc_time_ms and dt_str_to_utc_time_ms
 - Main now calls get_klines_cmd.
 ([3b0f621](3b0f621be34ce0ad52959962eafe492dcc2ec245))
- Add get_kline

Add it as a fn in binance_klines.rs and as to Opts.
 ([101976e](101976ed6f71bfac2c1f753b2f556480261ecada))
- Add binance_klines

The fn get_klines can be used to retrieve historical pricing information.

Added --get_klines (-K) option which right now returns the kline
associated with the current minute or the previous minute if less then
10 seconds of the current minute has elapsed.
 ([8e893e6](8e893e6bbab30fb2629601d141c4cfb565eeafca))
- Remove ie_new at end of main

I'd added just to validate it worked in something other than a test.
 ([eb26607](eb266074b9f3a6411705d2da05331a3ec10aa65f))
- Change BinanceContext parameters to use Option

This makes it clearer the things are optional rather than using empty
strings.

Also, the mutable reference to BinanceContext isn't needed not sure
why it ever was, but nice that's it's immutable now!
 ([b1ef68c](b1ef68c6c43f1d8e766edea3f1712afac276ab39))
- Move InternalErrorRec to common

And create ie_new macro
 ([bc6ed2c](bc6ed2c22a337124ac0b3bc41cbdd07020eff06d))
- Add HeaderRec and InternalErrorRec

Currently unused but there are some tests.
 ([8ab67e3](8ab67e3782eaa85c822bca9fb404998fdd1b63cb))
- Add support for all documented order responses

 - Add SuccessAck(AckTraceResposneRec)
 - Add SuccessResult(ResultTradeResonseRec)
 - Add SuccessUnknown(UnknownTradeResponseRec)
 ([b90cd13](b90cd13ac9d2ae43f8bf3c15808f9a61fb5ce4d5))
- Add --display_order_log

This displays the by using `--display_order_log file_name` or
the using the short option `-D file_name`
 ([333297a](333297af58e7691bc0e877cce2b465bf9f17ac89))
- Fix log_order_response

Previously I got an error the writer was moved in the
`serde_json::to_writer`line hence I couldn't have the
`writer.write_all(b"\n")?;` line in the code.

The solution was to pass `mut writer: &mut W` instead of just
`writer: &mut W`.
 ([cbe2455](cbe24553a292559ec222915af41a0891a20ef502))
- Fix lot size checking

Turns out LOT_SIZE is used instead of MARKET_LOT_SIZE as all
SizeRec.min_qty and step_size are zero for MarketLotSize in
ExchangeInfo.
 ([9089c16](9089c160cbcccdfe769c9a09df473697a96fd43e))
- Rename binance_sell_market to binance_market

 - Change sell_market to market_order in binance_market
 - Change Opts sell to sell_market and add buy_market
 - Cleanup Opts help
 ([335c80b](335c80b369b9676f66cbb4cd5e086dbad5010d93))
- Fix type in comment
 ([e10de86](e10de8620cc3d059dbd533be6472c4674758011d))
- Add get_my_trades

This does provide the "fill" information not provided by get_all_orders.
 ([ef31684](ef31684e5376e988c62f08c2b503953c5cf6fbf5))
- Add get_all_orders

 - Made symbols_map public in ExchangeInfo
 - Extraced orders_get_req_and_response out of get_open_orders
   and use it for both get_open_orders and get_all_orders
 - Add utc_to_time_ms to common
 - Handle get_all_orders, need to add more parameter options
 ([21385af](21385afbf2d336d815a7f0f59ce84bf63d23657c))
- Rename binance_open_orders to binance_orders

 - Rename OpenOrdersRec to OrdersRec
 - Rename OpenOrders to Orders
 ([405fc25](405fc25401e8ec12ede997ab1e32c380097c5c75))
- Refactoring

 - Add update_value AccountInfo
 - Use BTreeMap instead of AccontInfo as we want it sorted.
 - Add print and update_print to AccountInfo
 - auto_sell improve output and have it sorted with Selling then Keeping
 - main use update_and_print
 ([336ec1b](336ec1b90f203509ad6838e88dcd3e86fdd380f8))
- Change logging of order response

Allow writing to file or a buffer so it's "easier" to do unit tests. And
I learned that you don't want to enable serde-float in rust_decmail
as it introduces rounding errors.
 ([114b197](114b197485293b62be73320a50d71a82838f555b))
- Rename cost_basis_usd to value_usd
 ([324de5d](324de5d6da5450f69e60fa309e6c2c9b63886845))
- Calculate cost_basis_usd and commission_usd

 - Add cost_basis_usd and commission_usd to FullTradeResponseRec.
 - Add passing exchanging info to binance_new_order_or_test so we
   can get the quote_asset.
 - Fix sell_market to pass ei and test to false.
 - Add convert and convert_commission.
 ([b8fd898](b8fd898d0f545d49efa6a5361ee50fe89dc7a38d))
- Clean up use crate in binace_avg_price
 ([cf5d137](cf5d1374ad1f187c36cf15d95cd08a2a3fd7e550))
- Add get_req_get_response_ll and post_req_get_response_ll

These are not tied to binance and can be used with other rest API's.
In particular I did a simple test with a Covalent API:
  https://www.covalenthq.com/docs/api/#get-/v1/pricing/tickers/


  #[derive(Debug, Deserialize, Serialize)]
  struct CovalentPaginationRec {
      has_more: bool,
      page_number: i32,
      page_size: i32,
      total_count: i32,
  }

  #[derive(Debug, Deserialize, Serialize)]
  struct CovalentSpotPriceItemRec {
      contract_decimals: i32,
      contract_name: String,
      contract_ticker_symbol: String,
      contract_address: String,
      // The standard interface(s) supported for this token, eg: `ERC-20`.
      // The standard interface(s) supported for this token, e
      supports_erc: Option<Vec<String>>,
      logo_url: String,
      quote_rate: Decimal,
      rank: i32,
  }

  #[derive(Debug, Deserialize, Serialize)]
  struct CovalentSpotPriceRec {
      updated_at: String,
      items: Vec<CovalentSpotPriceItemRec>,
      pagination: CovalentPaginationRec,
  }
  #[derive(Debug, Deserialize, Serialize)]
  struct CovalentSpotPrices {
      data: CovalentSpotPriceRec,
      error: bool,
      error_message: Option<String>,
      error_code: Option<i32>,
  }

  #[allow(unused)]
  async fn covalent_spot_prices(
      api_key: &str,
      tickers: Vec<&str>,
      _page_number: Option<i32>,
      _page_size: Option<i32>,
      _format: Option<String>,
  ) -> Result<CovalentSpotPrices, Box<dyn std::error::Error>> {
      let mut csl = tickers[0].to_owned();
      for ticker in tickers.iter().skip(1) {
          csl.push_str("%2C"); // Escaped comma
          csl.push_str(&ticker);
      }

      let url = format!(
          "https://api.covalenthq.com/v1/pricing/tickers/?tickers={}&key={}",
          csl, api_key
      );
      trace!("url: {}", url);
      let response = get_req_get_response_ll(&url, HeaderMap::new()).await?;

      trace!("covalent_conv: response: {:#?}", response);
      let response_text = response.text().await?;
      let spot_prices: CovalentSpotPrices = serde_json::from_str(&response_text)?;
      trace!("covalent_spot_prices: {:?}", spot_prices);

      Ok(spot_prices)
  }

  async fn test_covalent_conv() {
      let api_key = "ckey_xxxx"; // A Covalent key
      let tickers = vec!["btc", "eth", "bnb"];
      let sp = covalent_spot_prices(api_key, tickers, None, None, None)
          .await
          .unwrap();
      println!("test_covalent_conv: sp: {:#?}", sp);
  }
 ([88b00cd](88b00cdebba2feb3951f6bdc6b827086d05c4173))
- Fix use order
 ([d3efb52](d3efb524c8accbcd00e32b813c5850f1542a201c))
- Improve testing

I couldn't use paramatertized test because the options processing by
BinanceContext::new() interferred with `cargo test` processing. The
solution was to use `cfg!(test)` in a conditional assignement and have
derive a Default trait for Opts.

See this discord post:
 https://discordapp.com/channels/273534239310479360/273541522815713281/840633869891797012
 ([b9fc3cb](b9fc3cbaac839ca67591e4cfeb654e2e1bf75b65))
- Change adj_quantity_verify_market_lot_size

Changing to "truncate" towards zero when adjusting the price so
we "always" sell slightlly less then the maximum since we don't
want to sell more than we have or to cross a "minimum".
 ([359cd24](359cd249f81852b81f6bb3145a4e055fa6b1a051))
- Add common post_req_get_response and get_req_get_response

This reduces duplicate code and allows for a single location so that
requests can be changed to use proxy.
 ([256257c](256257c530bf71e2ba3d99077d457ad861400aca))
- Change account_info to print sorted balances
 ([48465a3](48465a317560c085bbd65a3a688021fa1e775c0d))
- Add cargo-precommit and update README
 ([9cf2d88](9cf2d88bac3e32eeb7104b1b280c856d13f83cb4))
- Allow keys to be changed based on auto sell config

 - Create struct Keys added to BinanceContext
 - Make BinanceContext clonable.
 - Moved order_log_file and log_order_response to binance_trade
   creating OrderLogger.
 - Changed binance_trade, binance_open_orders and binance_auto_sell
   use BinanceContext.keys.
 ([a36a2ad](a36a2ad690dbe53fcf3c415f726140c1089888c3))
- Add logic declide what to keep or sell
 ([5dc5dfb](5dc5dfb7874309477ad9da797a2a339baa87e40b))
- Get AccountInfo in auto_sell

 - Usio tokio::fs instead of std::fs read_to_string
 - Diaplay asserts with non zoer free or locked balances
 ([84328ce](84328ceb744b597382eb288ef691868d54b32569))
- Convert auto_sell config_file to ConfigAutoSell

 - Also fix default_sell_to_asset typo
 ([cdae7b8](cdae7b86ed2bcd7976f38392677f03a54be82cbf))
- Add Opts.auto_sell

 - Rename binance_config_auto_sell to Binance_auto_sell
 - Add fn auto_sell and invoke if --auto-sell option is not empty
   from main. Currently does nothing except check params.
 ([04b3072](04b3072149ad47e4c9753f34b464c32ded786724))
- Rename binance_config_auto_sell_all

To binance_config_auto_sell because it doesn't sell everything as
typically they'll be assets that you'll want to "keep" as defined in
the "config-auto-sell.toml" file.
 ([4eee18f](4eee18fca20f0e58741b6beec22a97a996b842fc))
- Change sell_market to take &ExchangeInfo

And simplify main opts processing.
 ([e50f871](e50f87127533ffc8600720865287fd9a4848fd0b))
- Add fn print to AccountInfo

And remove it from main.
 ([c36a853](c36a853e597c0e32e541f68af2b74d95ebe70e64))
- Add binance_config_auto_sell_all

A toml configuration file for selling assets
 ([d46010e](d46010ee8fad6eebbbc3cfe0fc7efe92f413562f))
- Rename sig_key to secret_key

 - Only allow long name for api-key or secret-key parameters
 ([6cdc0eb](6cdc0ebe4f62dfd3e9fd566ab6232b785a7b5ca8))
- Create binance_sell_market

 - This is the code that was in main move to binance_sell_market.
 - Also changed the MarketQuantityType::Quantity to Decimal
 - Changed ctx have type &mut BinanceCointext (It has to be
   mutable because of `log_order_response`.
 ([1fd5cdb](1fd5cdbac143198bf1ee209e793ebc1c2fe7d33a))
- Add binance_verify_orders

 - Working at level 1
 - TODO: more tests
 ([4e4e305](4e4e305339f1b1588855fe3db50231063f416f6f))
- Add crate test-env-log

So I can see traces when running `cargo test`
 ([a066ec9](a066ec9c13b187abe4dc14b64e90e26da3b15d3b))
- Have git ignore /data/
 ([d635c34](d635c3440b63a3c6114f7de50e61a7d0286b6c38))
- Use get_open_orders
 ([8a22002](8a220024f09d321955ce4836afb1aad91e4a88aa))
- Fix miss use of symbol.base_asset

Change those to symbol.symbol
 ([ba69bbf](ba69bbf8a8c29cd319dd1e75698093e29b1a2ad7))
- Change binance_order_response to use Decimal

since all f64 types are now Decimal types I've added
allow(unused) to de_string_or_number_to_f64 as ATM it
has no users.
 ([0b27525](0b2752507021e6184d77fe8693312a5437ee3765))
- Change OpenOrderRec to use Decimal
 ([2606a6c](2606a6c75c381bff99e668f605f959cd12944d05))
- Change AccountInfo to use Decimal
 ([c6c72fb](c6c72fb5b5fa6fd8de55aeed8fdb8951abdf4a21))
- Change PriceFilterRec, PercentPriceRec to Decimal

Now there are no more f64's in binance_exchange_info
 ([86fb487](86fb4879a1468facf0efe9ad62df1ed25ff7d7c9))
- Change Opts.quantity to Decimal
 ([b02ff1f](b02ff1f7b7062401d2ccfc7a750fdbab6fba00eb))
- Change MinNotionalRec to Decimal
 ([0f1d8c4](0f1d8c4777dfe07127e513b349b7cb228489599f))
- Change MaxPosition to Decimal
 ([ed65ab3](ed65ab37d96c0330c83c666825cfb2c82b1d06ba))
- Use Decimal for AvgPrice
 ([9253520](92535202d0aa516c4caeeb05c1e5e08e8c3e4e0a))
- Remove commented out deserialize_with from SizeRec
 ([f319a5c](f319a5c16371f37acb458769fc3d6c7ca1b0ba87))
- Change Balance to use Decimal

Also total_value and price in main
 ([7df9352](7df9352795d4fbc48bdc5f110892cf14305cfebe))
- Use rust_decimal for MarketLotSize

And use rust_decimal for quantity when --sell
 ([9798a40](9798a405399a3bc9e85e76d30c85d143660f8c0c))
- Update dependencies

Ran `cargo update`
 ([7ae6074](7ae6074a3b03a2c4875b162becc1c1adf5a24d2a))
- Add binance_open_orders module

And --get_open_orders -O with optional symbol. If no symbol then all
open orders are returned.
 ([3e22ea2](3e22ea2270b13e0d903220aa3d8e1afbd396432b))
- Change get_exchange_info to allow proxy

And make return an error if response.status is not 200.
 ([8b9666f](8b9666f591d396cfa2ff8eed9051d0a839f131ba))
- Move OrderType and Side to common

 - Rename OrderResponse to TradeRespone
 - Rename TradeResponse::Success to SuccessFull in anticipation
   of SuccessResult and SuccessAck
 ([f5dc154](f5dc15424c31bb99d831770173362d7c3c3e2ec0))
- Verify that selling passes the required filters.

Still need to add getting open_orders
 ([1d08497](1d0849757a771a0d2ca32b333eb5d6b9994d86d1))
- Fix trace statement typo
 ([a26f1ab](a26f1ab3d6ce3c2a7950269da04d9887df533ac3))
- SymbolFilters records need public fields

Fix PriceFilterRec PercentPriceRec and MinNotionalRec
 ([1d25641](1d256411ff5668c2e8e97ac66bdbc590d4412d66))
- Change AccountInfo.balances

Change AccountInfo.balances: Vec<Balance> to
  AccountInfo.balances_map: HashMap<String, Balance>
 ([06090f5](06090f50e4d7c823ff99208d805d417474c15276))
- Handle errors and improve formatting

If unable to get  avgerage price in USD assume price is 0.
 ([19427cf](19427cf0eef5f4dc5d8c6ab9d25e885fc0aab789))
- Remove de_vec_to_hashmap

It served as the experiment for creating the de_vec_xxx_to_hashmap
functions in binance_exchange_info.rs

Also, reorganized the `mod` and `use` declarations in main.rs
 ([a5868d0](a5868d07486effd71c7a48473394a0976cd7a6ba))
- Rename order_response.rs to binance_order_response.rs
 ([aad8277](aad82779262bd415a3f154b2694a132d6129c16b))
- Rename exchange_info.rs binance_exchange_info.rs

 - Move get_exchange_info to binance_exchange_info.rs
 ([e3a30e3](e3a30e3f7bc74f7567bd7900ccb7b6f9b9934687))
- Rename account_info.rs to binance_account_info.rs

 - Move get_account_info into binance_accout_info.rs
 - Remove printing of keys and fix cli.rs tests
 ([44db5f5](44db5f52e5bbb7eb59b65447952d4174a1a738e3))
- Rename `rust-toolchain` to `rust-toolchain.toml`

Now that `rustup` 1.24.1 is release this now works and updated
README to reflect minimum `rustup` versionl
 ([a77b7b6](a77b7b6d926d0c3cb495f0f8b5aead731228d4ad))
- Improve error handling

Changed `struct BinanceResponseError` to `struct ResponseErrRec` and
then created `enum BinanceError` with a `Response` variant which has a
`ResponseErrorRec`.

And then have `BinanceError` implement `Error`. This allows returning
`Errr(BinanceError::Response)` as an `Error`:

        let response_error_rec =
            ResponseErrorRec::new(false, response_status.as_u16(), &url, &response_body);
        let binance_error_response = BinanceError::Response(response_error_rec);

        Err(binance_error_response.into())
 ([f855081](f855081da11258e8c9889eb1e84d0922ee81e956))
- Use determine value of each asset and the total value

Use get_avg_price to determine these when getting account info.
 ([b3d0eb3](b3d0eb35702f8e15fa8c18b1bf8380503d17e10f))
- Add binance_avg_price

 - Change OrderResponseFailure to BinanceRequestError and move it to
   common.
 - Change binance_trade to use BinanceRequestError and remove
   DEBUG println! and use trace! instead
 - Change BinanceContext::log_order_response to borrow order_response
   and return ()
 ([f00f4c7](f00f4c75868159b0f79fd787c518f1f119c6e5bd))
- Round up utc_now_to_time_ms

This will lower the maximum error from about 1ms to -0.5ms to +0.5ms.

Also, remove Utc_now_to_time_ns which I don't need and by having
utc_now_to_time_ms use Utc::now() directly Tarpaulin doesn't need to
instrumentment it and so the maximum time I see was reduced to just
over 2ms instead of 7ms.
 ([99f2686](99f268622639037a9ceb2db941350d972c45d1dd))
- Change test_utc_now_to_time_ms

There is no guarantee something could cause the loop to take much
longer, but it should never be shorter than 1ms.
 ([d88fcaf](d88fcaf9382b54f20511efdc389be346a2f837ee))
- Remove [all] section/table from tarpaulin.toml

Having a section causes tarpaulin to ignore some passed flags and options.
In particular `--skip-clean` is ignored.
 ([fb5ec0d](fb5ec0db46365cd81fe276f95b7a8342f67f8982))
- Add setting order log path

 - Add order_log_path option and ORDER_LOG_PATH environment variable
 - Rename order_logger.txt to order_log.txt
 - Delete data/order_logger.txt
 - Ignore data/order_log.txt
 ([839c27c](839c27c89a289f6fbfecd35dedabb6764609b8d0))
- Add BinanceContext.log_order_response
 ([d5a1db7](d5a1db79d572cc9d4d2a2ffcd81f556a38f315e5))
- Add order logging

 - Added BinanceContext.order_log_file: File.
 - Changed OrderResponse to OrderResponseSuccess.
 - Added OrderResponseFailure.
 - Change OrderResponse to be an enum with Success and Failure variants.
 - Changed binance_new_order_or_test to log all OrderResponse's.
 ([f247870](f247870e1c79a499edcd122ba0aa9f5f210b2171))
- Add binance_trade and order_response modules

Only support market selling at the moment.
 ([0bda5d6](0bda5d602f302ab7044041d003978febf22ea980))
- Move append_signature to binance_signature module
 ([2203ae5](2203ae5e150c6a4d5b20a5bed01cd109250eda01))
- Create common module

Has some time routines for now.
 ([7a5de71](7a5de71fff6957c6ade44de7530130cbf4001f2a))
- Cleanup timestamp_ms_to_secs_nsecs

I think this is a little clearer and informative.
 ([20b8ec6](20b8ec6d6ad63f04aab5cd40e9c8b1098c439861))
- Change get_account_info output

 - The output only for assests with balance.free > 0.0.
 - Added time_ms_to_utc
 - Added timestamp_ms_to_secs_nsecs and a test for it. Used
   by time_ms_to_utc.
 ([f417ab1](f417ab1b7cfd69384e8e8ea10de805729691612c))
- Add get_account_info
 ([9aca85f](9aca85f1c46a5a3fb97b60717f0524731188129e))
- Add binance_signature
 ([c87dcf2](c87dcf251380140bff6a132b7d33427e5a6b24c6))
- A little cleanup of account_info and exchange_info
 ([8c540e1](8c540e132ee3a1592bf74c5e79f52bf0553b5916))
- Add account_info.rs

This handles the response form a Binanace get current account
information command.
 ([e8fcb28](e8fcb28f36723af2626e82f6ae36a646ea8597ae))
- Add command line arguments

Cleanup command line handling moving it into binance_context.txt.
More tests are needed to get better coverage and added:

 - exchange-info
 - symbol

I was unable to continue to use clap v3.0.0-beta.2 it was unable to
handle optoinal string paramters, see comment at top of
binance_context.rs.
 ([139c463](139c4631c63daa6ff775800164a2fb202b090e7f))
- Rename ExchangeInfo.get_sym to get_symbol

And test it :)
 ([b2204bd](b2204bd529d60ed0359e27f90c93cdea5a373cb5))
- Add ExchangeFilter and RateLimits getters

More reorganizations and cleanup
 ([b04f910](b04f910c417b7c17c8ba580087bc50e637e02a63))
- Add get_max_position
 ([5b8f4df](5b8f4df3db66b4066d95dde25592ebd03c0dc5af))
- Add get_max_num_iceberg_orders

A little reorganization.
 ([4214e43](4214e436ddbc23e7b908554e083f3785b03dc634))
- Add get_man_num_algo_orders
 ([8737d46](8737d466302affdab6c7b3a741c9d2027c4d5fd5))
- Add get_max_num_orders
 ([8c21429](8c2142962a25f2ddeea9558c0c2836dfcc11df8d))
- Add get_iceberg_parts
 ([f8e42ff](f8e42ff1b017d440444106c56d14133143804338))
- Add get_min_notional

 - Add MinNotionalRec
 ([090626c](090626cc5d1c043cde1ce02ac8cb5bfc94ec5c6d))
- Add get_price_filter and get_percent_price

 - Add PriceFilterRec and PercentPriceRec
 ([615d6c5](615d6c563122270432159f1805426b6dc9784558))
- Add get_market_lot_size

 - Changed MarketLotSize to use SizeRec
 - Removed ExchangeInfo::get_lot_size is in efficient
   and probably not that useful.
 ([bc4d385](bc4d38569543a9aa3e0adf81ba44588225afea1b))
- Rename SizeData to SizeRec
 ([e5e3f98](e5e3f98ae69032ba27142abd10bdb4eef490892f))
- Add get_lot_size and tests
 ([b0a1280](b0a1280dc714536251ab084e44b567f8e5b974f2))
- Used clippy and fmt to cleanup the code
 ([e5a076d](e5a076d832d624a55ac6845fd3dfc3e59a6856ef))
- Have ExchangeInfo.symbol_map deserialize directly

 - Created dev_vec_symbols_to_hashmap
 - Updated tests
 ([c73752e](c73752e67f800e347aa39aca4a32435637a00176))
- Experiment with deserialize vector to hashmap
 ([5ef49c1](5ef49c1f6186fbf8dd071cd5bc7db15d46e35e52))
- Attempt to deserilize from json vector to HashMap.

ATM filter_map is being processed but not yet created.
 ([87eb0b9](87eb0b9593888369ace261be6b5d720e43ebbee3))
- Make test_create_filters

Instead of commenting it out make it a test :)
 ([fa4882f](fa4882f77d36f6153586d402d17aab1a7c121fbb))
- Move EXCHANGE_INFO_DATA

Move it to the end of the module to make looking at the `code` easier.
 ([f4fa3d7](f4fa3d774bf04c68303fd7c4d40bf7e50980ea44))
- Kinda doing something.

I can ExchangeInfo.create_symbols_map or Symbol.create_filters_map but
not both :(
 ([25a1c6b](25a1c6b3bd1083898e2adae57d148771b9b79f5b))
- WIP battling lifetimes for the first time :)

I "think" the problem is that the two HashMap values are
"reference/borrow" to members Symbol.symbols and SymbolFilters.filters.


Solutions?:
 * Use a reference count (Rc or Arc).
 * Copy the data Symbol and SymbolFilter data from the Vec to HashMap
 * Move the data Symbol and SymbolFilter data from the Vec to HashMap
 * ???
 ([63320b0](63320b0910073f72b52a513645b47701a730de32))
- Experiment with SymbolFilters

Seems using `struct SizeData` is better, but still need to probably
add a Symbol.get_lot_size to Symbol and have it use
SymbolFilters.get_lot_size?
 ([19c2df7](19c2df7eef269892a2d749d918dfc98d8d0a8177))
- Add ExchangeInfo.symbol_to_map

Made all struct fields public so they are accessible.
 ([bb0b2ac](bb0b2ac7c4e33671e1b22e553cf1e32ac15969c8))
- Add ExchangeInfo.symbol_to_map
 ([006cd9f](006cd9fbb87aa6f88285aaba77a57b32a97609db))
- Make struct fields public

So we can actually use their values.
 ([07521ac](07521acfe4c3062a9f0ff6592841674f3501175f))
- Create module binance_context
 ([741dbb1](741dbb17cdcd111e93d729a5fdf7a8c1e4398609))
- Add fn get_exchange_info

Using modules log, env_logger, reqwest and tokio
 ([3b72ca1](3b72ca12b7b84effa43236bfd84e7724c955955c))
- Update README.md
 ([87b0023](87b00235c5662e35ac090fe9734d83863f20256c))
- Add module exchange_info
 ([112d48c](112d48c54fef9ac720ceb308066ac2fd60587fe8))
- Add module de_string_or_numbers

The allows numeric string to be serialized to a number for i64, u64
and f64.

Added tarpaulin.toml so no parameters are needed for interactive
mode. And we'll see if they work for ci-stable.yml.
 ([a1a248a](a1a248ab322daab28abf6634862787ea2eda04a5))
- Update README.md

Add 100% line missing from tarpauling output
 ([afe3ac2](afe3ac232ddbf5b80368fd715534fde0a0cbdc6e))
- Simplify main

Adding `do_it()` isn't necessary to get "proper" code coverage numbers.
 ([231e7c9](231e7c91953b1aef23fa8ff75dec54657dad7502))
- Update README.md

Flesh out with new information especially what's needed for code coverage
and, in particular, the build problem problems that I found.
 ([55c0686](55c0686535219ae753c6befa6643431b1957bb3e))
- Remove timeout from tarpaulin command

Timeouts have been problematic with CI operations in general, so remove
for now and see if there is a problem.
 ([d9e600b](d9e600bc49593dd45a2077f2c4dac1d99dff58a7))
- Improve coverage report

Attempt to use `--follow-exec` parameter as mentioned in this issue:
 https://github.com/xd009642/tarpaulin/issues/704

Using it locally I now get the report I'd expect:
 $ cargo tarpaulin --follow-exec --verbose --all-features --workspace --timeout 120 --out Xml
 Apr 13 17:16:12.957 DEBUG cargo_tarpaulin: set up logging
 Apr 13 17:16:12.977  INFO cargo_tarpaulin: Running Tarpaulin
 Apr 13 17:16:12.977  INFO cargo_tarpaulin: Building project
     Finished test [unoptimized + debuginfo] target(s) in 0.01s
 Apr 13 17:16:13.036  INFO cargo_tarpaulin::process_handling::linux: Launching test
 Apr 13 17:16:13.036  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/cli-595a799ae4fee77f

 running 4 tests
 test test_no_params ... ok
 test test_help ... ok
 test test_req_params_as_env_vars ... ok
 test test_req_params ... ok

 test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.32s

 Apr 13 17:16:16.399  INFO cargo_tarpaulin::process_handling::linux: Launching test
 Apr 13 17:16:16.399  INFO cargo_tarpaulin::process_handling: running /home/wink/prgs/rust/projects/binance-auto-sell/target/debug/deps/binance_auto_sell-8b2f8d3614c3ece0

 running 0 tests

 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Apr 13 17:16:16.971  INFO cargo_tarpaulin::report: Coverage Results:
 || Uncovered Lines:
 || Tested/Total Lines:
 || src/main.rs: 6/6 +0%
 ||
 100.00% coverage, 6/6 lines covered, +0% change in coverage
 ([5221ab5](5221ab5a589802df9aa6c74b4674dbb3c9faf946))
- Try to improve code coverage report

Have Tarpaulin ignore main() and add fn do_it() instead.
 ([9ae94d5](9ae94d57bde52b76990a57d6f4e330a0bc734438))
- Update ci-stable.yml

 - Remove version from Run cargo-tarpaulin, it seems an odd value
   the version appears to be 0.1.3

 - Add token to Update to codecov.io. The docs say it isn't needed for
   public repos, but I'm getting an error:
    Please provide an upload token from codecov.io with valid arguments
 ([4f897ed](4f897ed1848560279eb84ac7150ded9715bbeed4))
- Add code coverage using Tarpaulin and codecov.io

Based on:
 https://github.com/actions-rs/tarpaulin#example-workflow
 ([52c9ec2](52c9ec24fcb65a536f9bcf15b15b0e40c120b0fb))
- Adding some command line processing

 - Require secret_key and api_key either as params or environment
   variables
 ([847d269](847d2694ff88755dcc98ef843bc824e1515b50ed))
- Add tests/cli.rs

Add testing command line output as per:
 https://rust-cli.github.io/book/tutorial/testing.html#testing-cli-applications-by-running-them
 ([d71a8f7](d71a8f74b66abf7b44e37228bdc28ac4d5ae8485))
- Add ci-stable.yml

We'll test both nightly and stable. Initially I thought I needed
to use the unstable `#[feature(test)]`, but since I'm not currently
planning on using `#[bench]` it's not needed.

Changed rust-toolchain to default to stable.
 ([adca6da](adca6da367ceac70adbde43e87294bcdf8a9687c))
- Add rust-toolchain and ci-nightl.yml

Trying to get github actions working.
 ([0650316](0650316f1b4ac55b0264d376b8e712c221518426))

## [0.1.0] - 2021-04-12

### Other

- Initial Commit

 - Ran `cargo new biance-auto-sell`
 - Added licenses and README.md
 ([a8c726d](a8c726d4734b7b1e42aa5ed5d9f68600f3eab970))

<!-- generated by git-cliff -->
