LD V0, 0        ; X
LD V1, 16       ; Y
LD I, DOT

MAIN:
    DRW V0, V1, 1
    DRW V0, V1, 1
    ADD V0, 1

    JP MAIN

DOT:
    DB 0xFF
