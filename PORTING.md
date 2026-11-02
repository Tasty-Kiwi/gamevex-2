# Porting roster

Priority agreed on 1 November 2026. Work stays in `gamevex-2`, directly on `main`, without pull requests. The original GameVex checkout is a read-only behavioral reference.

1. **Solitaire family parity — complete**
   - Relaxed FreeCell alongside Standard, with separate records and saved mode.
   - Safe automatic finishing for provably completed Klondike/FreeCell positions, with one-step undo.
   - Dealing, card movement, and exposed-card flip animations across Solitaire, FreeCell, and Spider.
2. **Hearts — complete**
   - Four-seat matches with three CPU opponents, rotating passes, legal-play guidance, moon scoring, and saved matches/records.
3. **Cribbage — complete**
   - Two-player matches to 121, CPU discards/pegging, go and last-card rules, ordered hand/crib scoring, saved progress and win counts.
4. **Texas Hold'em — complete**
   - No-limit hands against 1–4 CPUs, fixed blinds, legal betting and short all-ins, side/split pots, showdown evaluation, and saved bankroll.
5. **Roulette**
6. **Backgammon**

Minesweeper, Blackjack, Baccarat, Sudoku, Mahjong, Solitaire, FreeCell, Spider, Hearts, Cribbage, and Texas Hold'em are already playable. Card Lab remains an interaction sandbox.

## Removed from the port

Zip, Slots, Shogi, Arcana Hunt, Night Crown, Card Capital, and Craps are excluded from the roster.

Shared tickets/shop progression and distribution/accessibility work remain outside the game-port queue above.

## Completion checks

Each game or parity step needs rules coverage against the original implementation, save validation/migration, responsive mouse/touch/keyboard controls, reduced-motion and sound settings, and native plus browser verification. Save commands commit authoritative state immediately; animations are transient presentation only.
