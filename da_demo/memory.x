MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH (RX) : ORIGIN = 0x07FC0000, LENGTH = 32K
  RAM (!RX)  : ORIGIN = 0x07FC8000, LENGTH = 16K
}