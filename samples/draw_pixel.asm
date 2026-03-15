LD V0, 32       ; X
LD V1, 16       ; Y

LD I, PIXEL

DRW V0, V1, 1

LOOP:
    JP LOOP

PIXEL:
    DB 0x80
