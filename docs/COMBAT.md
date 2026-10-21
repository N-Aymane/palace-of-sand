# Sword fighting and parry

## Controls

- Find and pick up the sword with Shift in the campaign, or choose **Sword practice** / press **T** for an immediate armed duel.
- **Z / Shift**: strike. Hold to repeat; a tap during the last ten ticks of recovery queues a follow-up.
- **X**: raise defence. **Up** also defends when an armed guard is nearby, retaining its normal jump/climb role elsewhere.
- **Touch**: use the dedicated **ATTACK** and **PARRY** buttons. The action button still performs pickups/grabs outside fighting.
- Moving away from a nearby opponent retreats while keeping the blade facing them.

## Rules

The simulation runs at 60 Hz. A swing has 24 ticks of commitment and recovery, with a hit window from ticks 6 through 12. Each swing can damage one guard once, within 56 logical pixels and in front of the player. Both fighters have three health points. Attacks cannot cancel the player's damage recovery.

Holding defence blocks frontal strikes without damage. Raising it within the twelve ticks before impact produces a **PARRY!**, staggers the attacker for 36 ticks and extends its cooldown to 90 ticks. Continuing to hold defence remains a block; release and raise again to attempt another timed parry. A perfect parry gives time for a counterstrike. Defence requires a sword and grounded footing.

An enemy telegraphs a strike with a windup pose. Successful hits, blocks and parries have a metal sound and visible feedback. Struck enemies cancel pending strikes, and defeated enemies cannot attack.

These are deliberately accessible game rules, not measurements of the DOS combat tables.

## Verification

Native regression cases cover repeat attacks, one hit per swing, recovery buffering, a timed parry and counter, a held block, retreat facing, unarmed defence failure, hurt recovery, entering the strike's active window and dead guards. The full campaign input replay must still complete. Browser tests exercise the Sword practice entry, defence against a live guard and three repeated hits through both keyboard and actual Chromium touch events. WebKit retains its startup/layout/input checks.

Verification completed: CI run `39621192204` passed all 24 Rust/level tests and four browser scenarios.
