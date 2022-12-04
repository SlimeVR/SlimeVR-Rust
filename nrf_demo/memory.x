MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* BOOT : ORIGIN = 0x00000000, LENGTH = 4K */
  FLASH : ORIGIN = 0x00001000, LENGTH = 1M - 4K /* 4K for boot */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}
