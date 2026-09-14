# Scored 19x19 games used by the scoring tests

All games are played under Chinese (area) rules with a 7.5 komi and end with
a counted result in their `RE[]` property. They come from the AI section of
the `yenw/computer-go-dataset` collection on GitHub (game records from the Fox
Go Server and from AI tournaments); the original file names are given below.

Counted with every stone on the board (the game was played out, no dead
stones left):

| file | original |
|---|---|
| `fox_2018-06-13_evc_vs_jinmao_W+2.5.sgf` | `Jin Mao Ce Shi/2018.06.13 evc VS 金毛测试 (010001051).SGF` |
| `fox_2017-07-16_guaitong_vs_lilong_W+2.5.sgf` | `Li Long/2017.07.16 guaitong vs 骊龙 (62083799).sgf` |
| `fox_2018-04-21_coreaface_vs_bensondarr_W+0.5.sgf` | `BensonDarr/2018.04.21 coreaface VS BensonDarr (010001406).SGF` |

Counted after removing the dead stones found by `Game::dead_stones`:

| file | original |
|---|---|
| `alphago_zero_vs_alphago_lee_game4_W+0.5.sgf` | `AlphaGo Zero/Extended Data Figure 1 - AlphaGo Zero 20 block vs AlphaGo Lee/Game_004.sgf` |
| `world_ai_open_2018_final2_golaxy_vs_aq_W+0.5.sgf` | `The 2nd World AI Go Open 2018/F 2 Golaxy vs AQ.SGF` |
| `fox_2018-05-05_clover77_vs_jinmao_W+1.5.sgf` | `Jin Mao Ce Shi/2018.05.05 clover77 VS 金毛测试 (010001866).SGF` |
| `world_ai_open_2017_r9-2_abacus_vs_tianrang_B+1.5.sgf` | `The 1st World AI Go Open 2017/9-2 Abacus VS TianRang.sgf` |

The Fox exports write the komi as `KM[750]`; `Game::from_sgf` divides a komi
of 100 or more by 100.
