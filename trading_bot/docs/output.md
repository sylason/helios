commit e75688bdfa4f2a68892874c1024449dd5ee7f79c (HEAD)
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 23:34:21 2026 +0700

    docs(errors): record root cause and fix for mtf 15s cache panic

commit 76c5c20df6f9bbc137053cffa2c5d775e1716215
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 23:34:11 2026 +0700

    fix(helios): eliminate phantom macd/vol_ma mtf calculations and fix zero-width dna mappable

commit 0357d80c2027dd8cf26fbbb1c432e84006cddc4a
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 22:40:59 2026 +0700

    feat(bot_engine): bind account leverage and sizing constants in walk forward harness and update audit docs

commit d3a315d5559a9c90b75d5060e3de4b661acfb09a
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 22:40:48 2026 +0700

    refactor(helios): prune phantom genes to 108D, smooth atr step to 0.10 and decouple leverage into user settings

commit f0554cb19515c7efe9589b36fa061022a4d4e9ef
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 14:51:03 2026 +0700

    fix(helios): eliminate double tanh in reversal and anomaly scanners by reading raw z-score

commit 79279932cfad454171c2bf397bc7c15d3ba71205
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 14:09:41 2026 +0700

    fix(helios): resolve FLAW-3.3 simplex NaN defense and FLAW-4.4 false win cooldown with last_closed_trade

commit 70116b312a6fe401f8e8137710303a2935967918
Author: sylason <vietitcode@gmail.com>
Date:   Wed Sep 9 00:59:46 2026 +0700

    fix(helios): resolve FLAW-3.2 volume z-score and FLAW-4.1 arbiter dead zone with zero-guarded weighted geometric mean

commit df96ff6d37e842820a5b0694173d6b04df7dddfc
Author: sylason <vietitcode@gmail.com>
Date:   Tue Sep 8 21:59:32 2026 +0700

    feat(bot_engine): implement 48h wfe validation gate and update audit todos

commit 2152af87f5779875a354f9e25fa0725bc4357222
Author: sylason <vietitcode@gmail.com>
Date:   Tue Sep 8 21:59:20 2026 +0700

    feat(helios): add regime dynamic break-even dna and fix range_osc falling knife
