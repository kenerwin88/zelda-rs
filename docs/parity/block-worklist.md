# Frozen ROM block worklist

Current native main: `a28e327d05f5fb9a7408c5536da08ddbdbe7ae66`. Frozen cohort: **2,825 iterations**, original `07/00` + `09/00` missing cohort from `cost-pacing22/decomposition.json`, including rows closed by later pricing. Capture: reset through 1,585,000 hosts using the in-repo CPU model with `ZELDA3_LEDGER_BLOCK_PROFILE=1`, `ZELDA3_LEDGER_PROFILE_ONLY=1` and exactly those iteration IDs.

Every selected iteration/owner agrees with retained `cost-pacing10/source-missing/routine-profile.csv` on calls, inclusive and exclusive clocks. Every PC receipt sum independently reconciles to that retained exclusive clock total. Selected structural rows also agree exactly. The anchor is the retained profile, not the fresh capture. Interrupt, refresh and DMA stalls are excluded from instruction clocks. Source tail jumps retain their call-frame owner. Entry M/X variants stay separate.

**10,759 observed block/width rows**, 10,719 distinct owner/block entries; source 920,104,454, native 716,766,822, signed exclusive difference **203,337,632 master clocks**. Native profiles use main's CPU-only logical replay with `--seed-every 2000` and the identical frozen selection. This is instruction accounting, not A/V or physical-field parity proof.

**Status and unpriced-clock interpretation.** `marker` means measured marker clocks matched this source owner/block; declarations alone do not qualify. `legacy scalar` means the native owner has non-marker charges, whose PCs are not recorded. `none` means no matching measured marker and no scalar charge under this owner. An owner alias can still charge elsewhere. Width ambiguity is retained in JSON; an ambiguous marker is not assigned twice.

**Source clocks without block attribution (U)** below count max(source clocks − matched marker clocks, 0) per block; ambiguous marker attribution subtracts nothing. They are a block-conversion inventory, including existing legacy scalar pricing, **not additive missing instruction clocks**. Exact per-block unpriced clocks cannot be recovered from scalar routine receipts. No scalar budget is distributed across blocks or fitted to lag counts. The separate signed routine gap shows measured native coverage; a zero/negative gap is not a new pricing target. A marker clock match does not prove semantic equivalence. Distinct unresolved block entries (B) measure the required audit/conversion surface, not a promise that every entry needs a new charge.

| Highest source-clock blocks | Source clocks | U: without block attribution | Zero-charge owner clocks | Legacy scalar unresolved |
| --- | ---: | ---: | ---: | ---: |
| Top 50 | 331,722,386 | **331,722,386** | 0 | 331,722,386 |
| Top 200 | 544,803,862 | **544,803,862** | 16,193,018 | 528,610,844 |

## Top 200 blocks by executed source clocks

The ordering follows `rom_block_worklist.py` by executed source clocks. End addresses are exclusive; M/X are entry widths (8 or 16 bits). All clocks are master clocks.

| Rank | Block → end | M/X | Owner routine | Source clocks | Native status | Matched marker clocks |
| ---: | --- | --- | --- | ---: | --- | ---: |
| 1 | `$00:85FE` → `$00:865C` | 8/8 | `$00:85FC` | 25,611,450 | legacy scalar | 0 |
| 2 | `$0D:BB44` → `$0D:BB5B` | 16/8 | `$0D:BB0A` | 15,207,132 | legacy scalar | 0 |
| 3 | `$00:8420` → `$00:8489` | 8/8 | `$00:841E` | 14,808,650 | legacy scalar | 0 |
| 4 | `$00:8830` → `$00:8878` | 16/16 | `$00:882E` | 14,264,142 | legacy scalar | 0 |
| 5 | `$05:DFBD` → `$05:DFE1` | 16/16 | `$05:DF6C` | 12,465,858 | legacy scalar | 0 |
| 6 | `$06:E436` → `$06:E457` | 8/8 | `$06:E41E` | 11,694,006 | legacy scalar | 0 |
| 7 | `$07:D798` → `$07:D7D8` | 16/8 | `$07:D798` | 11,667,210 | legacy scalar | 0 |
| 8 | `$0D:BB0A` → `$0D:BB1E` | 8/8 | `$0D:BB0A` | 11,082,342 | legacy scalar | 0 |
| 9 | `$0D:FDE9` → `$0D:FDEF` | 16/16 | `$0D:FDD9` | 10,743,096 | legacy scalar | 0 |
| 10 | `$05:DF83` → `$05:DFA3` | 16/16 | `$05:DF6C` | 10,515,032 | legacy scalar | 0 |
| 11 | `$06:F7F4` → `$06:F831` | 8/8 | `$06:F7EF` | 8,932,272 | legacy scalar | 0 |
| 12 | `$06:84C1` → `$06:84DA` | 8/8 | `$06:84C1` | 8,898,280 | legacy scalar | 0 |
| 13 | `$06:E41E` → `$06:E436` | 8/8 | `$06:E41E` | 8,873,256 | legacy scalar | 0 |
| 14 | `$06:E89A` → `$06:E8BF` | 16/16 | `$06:E883` | 8,848,160 | legacy scalar | 0 |
| 15 | `$0D:FDB5` → `$0D:FDC0` | 16/16 | `$0D:FDAB` | 8,057,322 | legacy scalar | 0 |
| 16 | `$09:C710` → `$09:C72D` | 16/8 | `$09:C6F5` | 7,924,770 | legacy scalar | 0 |
| 17 | `$06:9271` → `$06:9283` | 8/8 | `$06:84E2` | 7,112,820 | legacy scalar | 0 |
| 18 | `$0D:F107` → `$0D:F10F` | 16/16 | `$0D:F0F7` | 6,603,142 | legacy scalar | 0 |
| 19 | `$07:D9E1` → `$07:DA00` | 8/8 | `$07:D9D8` | 6,573,096 | legacy scalar | 0 |
| 20 | `$06:E45F` → `$06:E476` | 8/8 | `$06:E41E` | 6,486,282 | legacy scalar | 0 |
| 21 | `$09:B0B6` → `$09:B0BF` | 8/8 | `$09:B0B6` | 6,340,596 | legacy scalar | 0 |
| 22 | `$06:F839` → `$06:F856` | 8/8 | `$06:F836` | 5,745,934 | legacy scalar | 0 |
| 23 | `$09:C739` → `$09:C74D` | 8/8 | `$09:C739` | 5,366,260 | legacy scalar | 0 |
| 24 | `$1B:BCD6` → `$1B:BCDE` | 16/16 | `$1B:BBF4` | 5,161,506 | legacy scalar | 0 |
| 25 | `$0D:FDAE` → `$0D:FDB5` | 16/16 | `$0D:FDAB` | 5,052,060 | legacy scalar | 0 |
| 26 | `$00:865C` → `$00:86EA` | 8/8 | `$00:85FC` | 5,014,802 | legacy scalar | 0 |
| 27 | `$05:DFA9` → `$05:DFB7` | 16/16 | `$05:DF6C` | 4,580,800 | legacy scalar | 0 |
| 28 | `$06:84AD` → `$06:84B9` | 8/8 | `$06:83F2` | 4,513,440 | legacy scalar | 0 |
| 29 | `$0D:BA96` → `$0D:BA9C` | 8/8 | `$0D:BA80` | 4,489,546 | legacy scalar | 0 |
| 30 | `$00:E843` → `$00:E84A` | 8/16 | `$00:E843` | 4,435,608 | legacy scalar | 0 |
| 31 | `$0D:F102` → `$0D:F107` | 16/16 | `$0D:F0F7` | 4,286,854 | legacy scalar | 0 |
| 32 | `$09:C702` → `$09:C710` | 16/8 | `$09:C6F5` | 4,207,010 | legacy scalar | 0 |
| 33 | `$07:DA1A` → `$07:DA2A` | 16/16 | `$07:D9D8` | 4,048,380 | legacy scalar | 0 |
| 34 | `$06:83F5` → `$06:8402` | 8/8 | `$06:83F2` | 4,045,662 | legacy scalar | 0 |
| 35 | `$09:C6F5` → `$09:C702` | 8/8 | `$09:C6F5` | 3,587,576 | legacy scalar | 0 |
| 36 | `$06:83A1` → `$06:83A7` | 8/8 | `$06:8328` | 3,525,600 | legacy scalar | 0 |
| 37 | `$07:DA04` → `$07:DA14` | 16/8 | `$07:D9D8` | 3,402,000 | legacy scalar | 0 |
| 38 | `$06:E740` → `$06:E75B` | 8/8 | `$06:E72F` | 3,297,276 | legacy scalar | 0 |
| 39 | `$0D:FDC0` → `$0D:FDC4` | 16/16 | `$0D:FDAB` | 3,197,350 | legacy scalar | 0 |
| 40 | `$06:DC30` → `$06:DC4C` | 8/8 | `$06:DC10` | 3,131,612 | legacy scalar | 0 |
| 41 | `$0D:A8D4` → `$0D:A92A` | 8/8 | `$0D:A18E` | 2,991,752 | legacy scalar | 0 |
| 42 | `$06:E75B` → `$06:E773` | 16/8 | `$06:E72F` | 2,943,006 | legacy scalar | 0 |
| 43 | `$0D:FDD9` → `$0D:FDDE` | 16/16 | `$0D:FDD9` | 2,919,854 | legacy scalar | 0 |
| 44 | `$06:DC13` → `$06:DC30` | 8/8 | `$06:DC10` | 2,859,672 | legacy scalar | 0 |
| 45 | `$09:B123` → `$09:B124` | 8/8 | `$09:B0B6` | 2,859,570 | legacy scalar | 0 |
| 46 | `$00:E84F` → `$00:E852` | 8/16 | `$00:E843` | 2,829,612 | legacy scalar | 0 |
| 47 | `$0D:BA9C` → `$0D:BA9E` | 8/8 | `$0D:BA80` | 2,739,384 | legacy scalar | 0 |
| 48 | `$1E:8B19` → `$1E:8B2E` | 8/8 | `$1E:8B19` | 2,698,740 | legacy scalar | 0 |
| 49 | `$06:83D7` → `$06:83E3` | 16/8 | `$06:83D3` | 2,559,450 | legacy scalar | 0 |
| 50 | `$1B:BCE6` → `$1B:BCEA` | 16/16 | `$1B:BBF4` | 2,523,156 | legacy scalar | 0 |
| 51 | `$08:832D` → `$08:8335` | 8/8 | `$08:832B` | 2,414,194 | legacy scalar | 0 |
| 52 | `$00:E7F2` → `$00:E7FC` | 8/16 | `$00:E75C` | 2,383,098 | legacy scalar | 0 |
| 53 | `$07:CE2F` → `$07:CE6A` | 16/8 | `$07:CE2A` | 2,360,156 | legacy scalar | 0 |
| 54 | `$06:E812` → `$06:E81F` | 8/8 | `$06:E72F` | 2,331,640 | legacy scalar | 0 |
| 55 | `$00:E7F2` → `$00:E7FC` | 8/16 | `$00:E766` | 2,315,364 | legacy scalar | 0 |
| 56 | `$09:B110` → `$09:B123` | 8/8 | `$09:B0B6` | 2,307,360 | legacy scalar | 0 |
| 57 | `$06:84E2` → `$06:84E7` | 8/8 | `$06:84E2` | 2,271,546 | legacy scalar | 0 |
| 58 | `$0D:A446` → `$0D:A483` | 8/8 | `$0D:A18E` | 2,254,014 | legacy scalar | 0 |
| 59 | `$0D:A48F` → `$0D:A49A` | 16/16 | `$0D:A18E` | 2,239,434 | legacy scalar | 0 |
| 60 | `$07:D0A2` → `$07:D0D4` | 16/8 | `$07:D077` | 2,218,338 | legacy scalar | 0 |
| 61 | `$09:C5E6` → `$09:C5F5` | 8/8 | `$09:C5BB` | 2,202,898 | legacy scalar | 0 |
| 62 | `$06:8417` → `$06:841E` | 8/8 | `$06:83F2` | 2,199,696 | legacy scalar | 0 |
| 63 | `$06:E72F` → `$06:E73C` | 8/8 | `$06:E72F` | 2,183,480 | legacy scalar | 0 |
| 64 | `$06:8496` → `$06:849C` | 8/8 | `$06:83F2` | 2,106,720 | legacy scalar | 0 |
| 65 | `$07:CDD0` → `$07:CE0F` | 16/8 | `$07:CDCB` | 2,073,866 | legacy scalar | 0 |
| 66 | `$02:F511` → `$02:F542` | 16/16 | `$02:F482` | 2,065,568 | legacy scalar | 0 |
| 67 | `$02:F511` → `$02:F542` | 16/16 | `$02:F50A` | 2,065,552 | legacy scalar | 0 |
| 68 | `$07:9873` → `$07:9896` | 8/8 | `$07:9873` | 2,013,498 | legacy scalar | 0 |
| 69 | `$0D:F122` → `$0D:F127` | 8/8 | `$0D:F0F7` | 1,981,824 | legacy scalar | 0 |
| 70 | `$0D:BB41` → `$0D:BB44` | 16/8 | `$0D:BB0A` | 1,969,766 | legacy scalar | 0 |
| 71 | `$06:8441` → `$06:8448` | 8/8 | `$06:83F2` | 1,965,232 | legacy scalar | 0 |
| 72 | `$0D:F11A` → `$0D:F120` | 8/8 | `$0D:F0F7` | 1,919,892 | legacy scalar | 0 |
| 73 | `$06:84E7` → `$06:84EB` | 8/8 | `$06:84E2` | 1,918,212 | legacy scalar | 0 |
| 74 | `$07:DC3F` → `$07:DC4A` | 16/16 | `$07:D9D8` | 1,873,310 | legacy scalar | 0 |
| 75 | `$06:84EB` → `$06:84F0` | 8/8 | `$06:84E2` | 1,856,682 | legacy scalar | 0 |
| 76 | `$07:D9D8` → `$07:D9DE` | 16/16 | `$07:D9D8` | 1,856,350 | legacy scalar | 0 |
| 77 | `$0D:A94B` → `$0D:A979` | 8/8 | `$0D:A18E` | 1,824,950 | legacy scalar | 0 |
| 78 | `$08:A7D7` → `$08:A802` | 8/8 | `$08:A7AB` | 1,804,082 | legacy scalar | 0 |
| 79 | `$06:E886` → `$06:E88E` | 8/8 | `$06:E883` | 1,792,834 | legacy scalar | 0 |
| 80 | `$01:FF28` → `$01:FFB6` | 8/8 | `$01:FF28` | 1,776,000 | legacy scalar | 0 |
| 81 | `$09:C625` → `$09:C634` | 8/8 | `$09:C5FA` | 1,771,114 | legacy scalar | 0 |
| 82 | `$09:B5DE` → `$09:B5F1` | 8/8 | `$09:B5DE` | 1,764,202 | legacy scalar | 0 |
| 83 | `$09:B091` → `$09:B096` | 8/8 | `$09:B08C` | 1,754,658 | legacy scalar | 0 |
| 84 | `$06:E9CC` → `$06:E9D5` | 8/8 | `$06:E991` | 1,748,994 | legacy scalar | 0 |
| 85 | `$0D:FC2C` → `$0D:FC5E` | 16/16 | `$0D:FB94` | 1,745,232 | legacy scalar | 0 |
| 86 | `$00:874E` → `$00:8781` | 16/8 | `$00:85FC` | 1,723,250 | legacy scalar | 0 |
| 87 | `$06:FF56` → `$06:FF6D` | 8/16 | `$06:FF56` | 1,717,700 | legacy scalar | 0 |
| 88 | `$08:A7B2` → `$08:A7D7` | 8/8 | `$08:A7AB` | 1,674,806 | legacy scalar | 0 |
| 89 | `$1E:FF39` → `$1E:FF51` | 8/8 | `$1E:FF34` | 1,663,922 | none | 0 |
| 90 | `$02:F449` → `$02:F47A` | 16/16 | `$02:F3B9` | 1,638,996 | legacy scalar | 0 |
| 91 | `$02:F449` → `$02:F47A` | 16/16 | `$02:F435` | 1,638,996 | legacy scalar | 0 |
| 92 | `$0E:D822` → `$0E:D832` | 16/8 | `$0E:D7FE` | 1,618,740 | legacy scalar | 0 |
| 93 | `$07:E3A4` → `$07:E3C1` | 8/8 | `$07:E245` | 1,616,104 | legacy scalar | 0 |
| 94 | `$06:F23A` → `$06:F241` | 8/8 | `$06:F228` | 1,615,454 | legacy scalar | 0 |
| 95 | `$0E:D80F` → `$0E:D81F` | 16/8 | `$0E:D7FE` | 1,614,828 | legacy scalar | 0 |
| 96 | `$06:83A7` → `$06:83AA` | 8/8 | `$06:8328` | 1,610,250 | legacy scalar | 0 |
| 97 | `$00:8784` → `$00:879C` | 8/8 | `$1E:8B19` | 1,604,022 | legacy scalar | 0 |
| 98 | `$02:BD78` → `$02:BDA5` | 16/8 | `$02:BD62` | 1,595,674 | legacy scalar | 0 |
| 99 | `$09:B5F1` → `$09:B601` | 8/8 | `$09:B5DE` | 1,592,190 | legacy scalar | 0 |
| 100 | `$07:CE8A` → `$07:CEBA` | 16/8 | `$07:CE85` | 1,590,450 | legacy scalar | 0 |
| 101 | `$06:84A4` → `$06:84AA` | 8/8 | `$06:83F2` | 1,579,704 | legacy scalar | 0 |
| 102 | `$06:8413` → `$06:8417` | 8/8 | `$06:83F2` | 1,579,140 | legacy scalar | 0 |
| 103 | `$0D:A9ED` → `$0D:AA18` | 16/16 | `$0D:A18E` | 1,570,700 | legacy scalar | 0 |
| 104 | `$09:B099` → `$09:B09C` | 8/8 | `$09:B097` | 1,567,680 | legacy scalar | 0 |
| 105 | `$09:B08E` → `$09:B091` | 8/8 | `$09:B08C` | 1,564,230 | legacy scalar | 0 |
| 106 | `$07:DC2E` → `$07:DC39` | 8/8 | `$07:D9D8` | 1,557,330 | legacy scalar | 0 |
| 107 | `$00:EDCC` → `$00:EDED` | 16/16 | `$00:EDCA` | 1,519,920 | legacy scalar | 0 |
| 108 | `$06:8439` → `$06:843E` | 8/8 | `$06:83F2` | 1,517,694 | legacy scalar | 0 |
| 109 | `$06:8514` → `$06:8520` | 8/8 | `$06:84E2` | 1,516,768 | legacy scalar | 0 |
| 110 | `$06:8431` → `$06:8436` | 8/8 | `$06:83F2` | 1,515,840 | legacy scalar | 0 |
| 111 | `$06:849C` → `$06:84A1` | 8/8 | `$06:83F2` | 1,514,466 | legacy scalar | 0 |
| 112 | `$05:CA3F` → `$05:CA64` | 8/8 | `$05:CA09` | 1,513,812 | legacy scalar | 0 |
| 113 | `$06:8429` → `$06:842E` | 8/8 | `$06:83F2` | 1,511,592 | legacy scalar | 0 |
| 114 | `$05:FF06` → `$05:FF2D` | 8/16 | `$05:FE6F` | 1,488,144 | none | 0 |
| 115 | `$0D:FC5E` → `$0D:FC8E` | 8/8 | `$0D:FB94` | 1,479,776 | legacy scalar | 0 |
| 116 | `$06:8421` → `$06:8426` | 8/8 | `$06:83F2` | 1,472,760 | legacy scalar | 0 |
| 117 | `$02:F4A5` → `$02:F4C5` | 16/16 | `$02:F482` | 1,459,120 | legacy scalar | 0 |
| 118 | `$06:E893` → `$06:E89A` | 16/16 | `$06:E883` | 1,453,050 | legacy scalar | 0 |
| 119 | `$0D:BA80` → `$0D:BA84` | 8/8 | `$0D:BA80` | 1,445,786 | legacy scalar | 0 |
| 120 | `$0D:A833` → `$0D:A844` | 8/16 | `$0D:A18E` | 1,444,320 | legacy scalar | 0 |
| 121 | `$08:8EC0` → `$08:8EEC` | 8/8 | `$08:8EAD` | 1,433,250 | none | 0 |
| 122 | `$06:E740` → `$06:E75B` | 8/8 | `$06:E73C` | 1,428,252 | legacy scalar | 0 |
| 123 | `$1D:C222` → `$1D:C237` | 8/8 | `$1D:C222` | 1,413,460 | legacy scalar | 0 |
| 124 | `$02:87E5` → `$02:881E` | 8/8 | `$00:80B5` | 1,411,380 | legacy scalar | 0 |
| 125 | `$0D:FB94` → `$0D:FBC0` | 16/16 | `$0D:FB94` | 1,406,352 | legacy scalar | 0 |
| 126 | `$06:F70A` → `$06:F72A` | 8/8 | `$06:F70A` | 1,406,240 | none | 0 |
| 127 | `$09:B104` → `$09:B110` | 8/8 | `$09:B0B6` | 1,372,996 | legacy scalar | 0 |
| 128 | `$0D:A1EE` → `$0D:A215` | 8/8 | `$0D:A18E` | 1,372,950 | legacy scalar | 0 |
| 129 | `$00:EE23` → `$00:EE44` | 16/16 | `$00:EE21` | 1,366,728 | legacy scalar | 0 |
| 130 | `$0D:A7DD` → `$0D:A802` | 8/16 | `$0D:A18E` | 1,335,760 | legacy scalar | 0 |
| 131 | `$02:F3DB` → `$02:F403` | 16/16 | `$02:F3B9` | 1,335,316 | legacy scalar | 0 |
| 132 | `$0D:A98C` → `$0D:A9B3` | 8/16 | `$0D:A18E` | 1,327,750 | legacy scalar | 0 |
| 133 | `$0E:D7FE` → `$0E:D80C` | 16/8 | `$0E:D7FE` | 1,314,198 | legacy scalar | 0 |
| 134 | `$1D:9FD7` → `$1D:A011` | 8/8 | `$1D:9F64` | 1,309,680 | legacy scalar | 0 |
| 135 | `$06:83F2` → `$06:83F5` | 8/8 | `$06:83F2` | 1,297,614 | legacy scalar | 0 |
| 136 | `$07:E3E0` → `$07:E406` | 8/8 | `$07:E245` | 1,284,738 | legacy scalar | 0 |
| 137 | `$06:E45F` → `$06:E476` | 8/8 | `$06:E41A` | 1,283,382 | legacy scalar | 0 |
| 138 | `$05:B5D3` → `$05:B5E8` | 8/8 | `$05:B5D3` | 1,275,710 | legacy scalar | 0 |
| 139 | `$0D:FCB2` → `$0D:FCDD` | 8/8 | `$0D:FB94` | 1,266,440 | legacy scalar | 0 |
| 140 | `$0D:F10F` → `$0D:F114` | 16/16 | `$0D:F0F7` | 1,259,284 | legacy scalar | 0 |
| 141 | `$06:F121` → `$06:F127` | 8/8 | `$06:F121` | 1,258,470 | none | 0 |
| 142 | `$06:E75B` → `$06:E773` | 16/8 | `$06:E73C` | 1,253,364 | legacy scalar | 0 |
| 143 | `$06:F856` → `$06:F85F` | 8/8 | `$06:F836` | 1,251,588 | legacy scalar | 0 |
| 144 | `$1D:E962` → `$1D:E97A` | 8/8 | `$1D:E95D` | 1,239,950 | none | 0 |
| 145 | `$09:B09C` → `$09:B09F` | 8/8 | `$09:B097` | 1,213,248 | legacy scalar | 0 |
| 146 | `$09:B787` → `$09:B78C` | 8/8 | `$09:B77E` | 1,212,006 | legacy scalar | 0 |
| 147 | `$06:E7A1` → `$06:E7A9` | 8/8 | `$06:E72F` | 1,205,118 | legacy scalar | 0 |
| 148 | `$06:F14A` → `$06:F154` | 8/8 | `$06:F145` | 1,201,980 | legacy scalar | 0 |
| 149 | `$09:B42C` → `$09:B446` | 8/8 | `$09:B0B6` | 1,199,424 | legacy scalar | 0 |
| 150 | `$0D:A80E` → `$0D:A831` | 16/16 | `$0D:A18E` | 1,194,160 | legacy scalar | 0 |
| 151 | `$07:E3C5` → `$07:E3D5` | 8/8 | `$07:E245` | 1,138,034 | legacy scalar | 0 |
| 152 | `$0D:F0F7` → `$0D:F102` | 16/16 | `$0D:F0F7` | 1,135,420 | legacy scalar | 0 |
| 153 | `$0D:FC8E` → `$0D:FCB2` | 8/8 | `$0D:FB94` | 1,118,304 | legacy scalar | 0 |
| 154 | `$1E:8B11` → `$1E:8B17` | 8/8 | `$1E:8B11` | 1,098,108 | legacy scalar | 0 |
| 155 | `$06:E89A` → `$06:E8BF` | 16/16 | `$06:E886` | 1,088,000 | none | 0 |
| 156 | `$06:DBF0` → `$06:DBF6` | 8/8 | `$06:DBF0` | 1,075,452 | none | 0 |
| 157 | `$09:B60B` → `$09:B613` | 8/8 | `$09:B5DE` | 1,075,410 | legacy scalar | 0 |
| 158 | `$08:F712` → `$08:F726` | 16/8 | `$08:F702` | 1,053,378 | none | 0 |
| 159 | `$06:E476` → `$06:E485` | 8/8 | `$06:E41E` | 1,052,272 | legacy scalar | 0 |
| 160 | `$06:E45F` → `$06:E476` | 8/8 | `$06:DC10` | 1,041,144 | legacy scalar | 0 |
| 161 | `$05:DFEC` → `$05:DFFA` | 8/8 | `$05:DFE9` | 1,032,876 | legacy scalar | 0 |
| 162 | `$0D:FBC0` → `$0D:FBE4` | 16/16 | `$0D:FB94` | 1,015,108 | legacy scalar | 0 |
| 163 | `$08:F671` → `$08:F6A4` | 8/8 | `$08:F671` | 1,013,642 | legacy scalar | 0 |
| 164 | `$00:E82E` → `$00:E840` | 8/16 | `$00:E75C` | 1,009,214 | legacy scalar | 0 |
| 165 | `$0E:D835` → `$0E:D83A` | 16/8 | `$0E:D7FE` | 1,007,424 | legacy scalar | 0 |
| 166 | `$07:DE67` → `$07:DE70` | 16/16 | `$07:D9D8` | 1,001,484 | legacy scalar | 0 |
| 167 | `$08:8338` → `$08:833B` | 8/8 | `$08:832B` | 1,000,050 | legacy scalar | 0 |
| 168 | `$09:B4C9` → `$09:B4FB` | 8/8 | `$09:B0B6` | 992,592 | legacy scalar | 0 |
| 169 | `$07:86A4` → `$07:86AB` | 8/8 | `$07:8689` | 988,750 | legacy scalar | 0 |
| 170 | `$06:E9D9` → `$06:E9DE` | 8/8 | `$06:E991` | 983,580 | legacy scalar | 0 |
| 171 | `$02:A47E` → `$02:A4B5` | 8/8 | `$00:80B5` | 972,288 | legacy scalar | 0 |
| 172 | `$07:8000` → `$07:8019` | 8/8 | `$07:8000` | 960,746 | legacy scalar | 0 |
| 173 | `$06:E7D8` → `$06:E7DE` | 8/8 | `$06:E72F` | 957,702 | legacy scalar | 0 |
| 174 | `$06:E878` → `$06:E87B` | 8/8 | `$06:E72F` | 948,024 | legacy scalar | 0 |
| 175 | `$06:DCCF` → `$06:DCEE` | 8/8 | `$06:DC10` | 938,308 | legacy scalar | 0 |
| 176 | `$05:FA05` → `$05:FA1D` | 8/8 | `$05:FA00` | 935,758 | none | 0 |
| 177 | `$0D:A8A9` → `$0D:A8C6` | 8/8 | `$0D:A18E` | 932,750 | legacy scalar | 0 |
| 178 | `$0D:FBED` → `$0D:FC09` | 8/8 | `$0D:FB94` | 931,920 | legacy scalar | 0 |
| 179 | `$09:C730` → `$09:C731` | 8/8 | `$09:C6F5` | 918,456 | legacy scalar | 0 |
| 180 | `$06:FF49` → `$06:FF56` | 8/16 | `$06:FF49` | 907,100 | legacy scalar | 0 |
| 181 | `$0D:A7B0` → `$0D:A7CE` | 8/16 | `$0D:A18E` | 906,240 | legacy scalar | 0 |
| 182 | `$06:E485` → `$06:E48D` | 8/8 | `$06:E41E` | 905,544 | legacy scalar | 0 |
| 183 | `$06:84AA` → `$06:84AD` | 8/8 | `$06:83F2` | 902,688 | legacy scalar | 0 |
| 184 | `$06:F241` → `$06:F244` | 8/8 | `$06:F228` | 899,034 | legacy scalar | 0 |
| 185 | `$06:DCCF` → `$06:DCEE` | 8/8 | `$06:DC64` | 898,380 | none | 0 |
| 186 | `$06:F2AA` → `$06:F2B0` | 8/8 | `$06:F2AA` | 893,260 | none | 0 |
| 187 | `$07:CECE` → `$07:CEFB` | 16/8 | `$07:CEC9` | 885,816 | none | 0 |
| 188 | `$06:F862` → `$06:F864` | 8/8 | `$06:F836` | 882,910 | legacy scalar | 0 |
| 189 | `$09:C76F` → `$09:C770` | 8/8 | `$09:C739` | 880,530 | legacy scalar | 0 |
| 190 | `$06:EAE8` → `$06:EB08` | 8/8 | `$06:EAE8` | 880,266 | legacy scalar | 0 |
| 191 | `$06:D312` → `$06:D33D` | 8/8 | `$06:D22F` | 872,998 | none | 0 |
| 192 | `$01:CEA0` → `$01:CEC9` | 8/8 | `$01:CE70` | 869,722 | legacy scalar | 0 |
| 193 | `$0D:A804` → `$0D:A80E` | 16/16 | `$0D:A18E` | 863,760 | legacy scalar | 0 |
| 194 | `$0D:A1D1` → `$0D:A1EC` | 8/8 | `$0D:A18E` | 863,454 | legacy scalar | 0 |
| 195 | `$05:CB7C` → `$05:CBA5` | 8/8 | `$05:CB64` | 859,974 | legacy scalar | 0 |
| 196 | `$1B:BBF4` → `$1B:BC2C` | 8/8 | `$1B:BBF4` | 858,972 | legacy scalar | 0 |
| 197 | `$0D:A49A` → `$0D:A49E` | 16/16 | `$0D:A18E` | 855,852 | legacy scalar | 0 |
| 198 | `$06:E8BF` → `$06:E8CE` | 16/16 | `$06:E883` | 845,910 | legacy scalar | 0 |
| 199 | `$00:E493` → `$00:E4A7` | 16/16 | `$00:E3FA` | 843,264 | legacy scalar | 0 |
| 200 | `$0D:A9B3` → `$0D:A9CA` | 16/16 | `$0D:A18E` | 841,850 | legacy scalar | 0 |

## Top 60 routines by positive exclusive deficit per distinct block

Sorted by max(source − native, 0)/B, then positive deficit, then source owner. Counts deduplicate entry PCs within each owner; M/X variants remain available in JSON. Include scalar-priced routines for marker-conversion planning, and consult the signed gap before considering any additional instruction charge.

| Rank | Owner routine | U: without block attribution clocks | B: distinct blocks | Positive deficit/B | Native exclusive | Signed source − native |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | `$06:F70A` | 1,406,240 | 1 | 1,406,240.0 | 0 | +1,406,240 |
| 2 | `$06:F121` | 2,026,350 | 2 | 1,013,175.0 | 0 | +2,026,350 |
| 3 | `$06:DBF0` | 1,731,660 | 2 | 865,830.0 | 0 | +1,731,660 |
| 4 | `$0E:DE9A` | 838,656 | 1 | 838,656.0 | 0 | +838,656 |
| 5 | `$06:F2AA` | 1,650,260 | 2 | 825,130.0 | 0 | +1,650,260 |
| 6 | `$06:E496` | 1,566,876 | 2 | 783,438.0 | 0 | +1,566,876 |
| 7 | `$0D:B871` | 774,330 | 1 | 774,330.0 | 0 | +774,330 |
| 8 | `$06:DC10` | 11,360,124 | 15 | 720,691.9 | 549,746 | +10,810,378 |
| 9 | `$1E:FF34` | 3,541,720 | 5 | 708,344.0 | 0 | +3,541,720 |
| 10 | `$08:8EAD` | 1,835,734 | 3 | 611,911.3 | 0 | +1,835,734 |
| 11 | `$08:F702` | 2,201,964 | 4 | 550,491.0 | 0 | +2,201,964 |
| 12 | `$09:B70C` | 537,200 | 1 | 537,200.0 | 0 | +537,200 |
| 13 | `$08:FADD` | 520,676 | 1 | 520,676.0 | 0 | +520,676 |
| 14 | `$1E:FF28` | 948,562 | 2 | 474,281.0 | 0 | +948,562 |
| 15 | `$00:8781` | 517,244 | 1 | 472,940.0 | 44,304 | +472,940 |
| 16 | `$1D:E95D` | 2,309,704 | 5 | 461,940.8 | 0 | +2,309,704 |
| 17 | `$06:F705` | 814,504 | 2 | 407,252.0 | 0 | +814,504 |
| 18 | `$06:E41A` | 2,780,416 | 4 | 406,553.5 | 1,154,202 | +1,626,214 |
| 19 | `$1E:8A85` | 385,308 | 1 | 385,308.0 | 0 | +385,308 |
| 20 | `$1E:FE78` | 2,625,028 | 7 | 375,004.0 | 0 | +2,625,028 |
| 21 | `$05:FA00` | 1,840,136 | 5 | 368,027.2 | 0 | +1,840,136 |
| 22 | `$05:FAA2` | 367,650 | 1 | 367,650.0 | 0 | +367,650 |
| 23 | `$07:983A` | 732,160 | 2 | 366,080.0 | 0 | +732,160 |
| 24 | `$06:E886` | 1,458,000 | 4 | 364,500.0 | 0 | +1,458,000 |
| 25 | `$06:DC64` | 2,783,576 | 8 | 347,947.0 | 0 | +2,783,576 |
| 26 | `$05:DF6C` | 30,691,084 | 11 | 340,100.2 | 26,949,982 | +3,741,102 |
| 27 | `$06:E932` | 673,552 | 2 | 336,776.0 | 0 | +673,552 |
| 28 | `$06:83EA` | 660,060 | 2 | 330,030.0 | 0 | +660,060 |
| 29 | `$07:CEC9` | 1,293,182 | 4 | 323,295.5 | 0 | +1,293,182 |
| 30 | `$06:E93E` | 1,492,192 | 5 | 298,438.4 | 0 | +1,492,192 |
| 31 | `$06:EAE8` | 986,288 | 3 | 292,620.0 | 108,428 | +877,860 |
| 32 | `$08:F6E1` | 1,354,380 | 5 | 269,628.0 | 6,240 | +1,348,140 |
| 33 | `$05:FE6F` | 1,600,368 | 6 | 266,728.0 | 0 | +1,600,368 |
| 34 | `$06:DC54` | 524,780 | 2 | 262,390.0 | 0 | +524,780 |
| 35 | `$06:E92C` | 1,784,758 | 7 | 254,965.4 | 0 | +1,784,758 |
| 36 | `$1D:E8A2` | 1,736,496 | 7 | 248,070.9 | 0 | +1,736,496 |
| 37 | `$07:B9B3` | 975,264 | 4 | 243,816.0 | 0 | +975,264 |
| 38 | `$1E:C267` | 709,484 | 3 | 236,494.7 | 0 | +709,484 |
| 39 | `$06:F129` | 468,160 | 2 | 234,080.0 | 0 | +468,160 |
| 40 | `$1E:8B2E` | 466,960 | 2 | 233,480.0 | 0 | +466,960 |
| 41 | `$05:925B` | 1,141,544 | 5 | 228,308.8 | 0 | +1,141,544 |
| 42 | `$1D:E952` | 447,664 | 2 | 223,832.0 | 0 | +447,664 |
| 43 | `$06:E96C` | 661,394 | 3 | 220,464.7 | 0 | +661,394 |
| 44 | `$06:F1F6` | 859,990 | 4 | 214,997.5 | 0 | +859,990 |
| 45 | `$1E:FF62` | 642,738 | 3 | 214,246.0 | 0 | +642,738 |
| 46 | `$05:F9F4` | 392,490 | 2 | 196,245.0 | 0 | +392,490 |
| 47 | `$06:E87B` | 380,000 | 2 | 190,000.0 | 0 | +380,000 |
| 48 | `$1E:FF21` | 560,254 | 3 | 186,751.3 | 0 | +560,254 |
| 49 | `$0E:DE49` | 921,600 | 5 | 184,320.0 | 0 | +921,600 |
| 50 | `$06:EAD1` | 855,464 | 3 | 177,449.3 | 323,116 | +532,348 |
| 51 | `$07:82DA` | 349,746 | 2 | 174,873.0 | 0 | +349,746 |
| 52 | `$06:DBF8` | 344,090 | 2 | 172,045.0 | 0 | +344,090 |
| 53 | `$07:B97C` | 1,195,940 | 7 | 170,848.6 | 0 | +1,195,940 |
| 54 | `$07:CF12` | 1,126,666 | 7 | 160,952.3 | 0 | +1,126,666 |
| 55 | `$1A:FDB5` | 1,113,780 | 7 | 159,111.4 | 0 | +1,113,780 |
| 56 | `$06:83E6` | 316,980 | 2 | 158,490.0 | 0 | +316,980 |
| 57 | `$06:FB2A` | 1,223,848 | 8 | 152,981.0 | 0 | +1,223,848 |
| 58 | `$0D:C0A6` | 855,652 | 6 | 142,608.7 | 0 | +855,652 |
| 59 | `$1D:8549` | 283,504 | 2 | 141,752.0 | 0 | +283,504 |
| 60 | `$06:DC5C` | 279,490 | 2 | 139,745.0 | 0 | +279,490 |

## Evidence and reproduction

Artifacts are under `target/block-worklist/` (untracked): `block-worklist.json` contains all enriched block rows, routine rollups, definitions and top-50/200 sums; `ranked-blocks.json` is the unmodified tool output; `source/instruction-profile.csv` contains the source PC receipts; `source/routine-profile.csv` and `source/structural-profile.csv` retain the recapture. `native-profile.csv` and `native-profile.blocks.csv` bind current-main native charges. `input-provenance.json` and `provenance.json` bind inputs, binaries and outputs by SHA-256.

```sh
python3 scripts/rom_block_worklist.py target/block-worklist/source/instruction-profile.csv \
  /Users/missingno/Documents/zelda3-rs-codex2/target/cost-pacing10/source-missing/routine-profile.csv \
  --cohort target/block-worklist/cohort.json \
  --native target/block-worklist/native-profile.blocks.csv \
  --json target/block-worklist/ranked-blocks.json --show 200
```

Source-profiler and default builds are warning-free. Existing block-cost/worklist (6), routine-coverage (4) and pacing-shortfall (6) Python tests pass. The current default binary matches all records and EOF in a fresh 200k main-binary state ledger. The worklist partitioner now keeps a shared successor separate when REP/SEP converge different entry widths; its regression verifies that the shared tail is counted once. No runtime, generated price table or pricing site changed; default/ROM-timing accounting is unchanged. No GPU execution, video, cached A/V, Snes9x execution, cargo GPU test, oracle-cache write, cargo fmt, commit or push was used. Supplied ledgers/cache and the other checkout stayed read-only.
