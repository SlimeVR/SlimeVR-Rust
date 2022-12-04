MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* BOOT : ORIGIN = 0x00000000, LENGTH = 4K */
  /* softdevice normally is at 0x1000, but we dont use that so we just write over it */
  FLASH : ORIGIN = 0x00000000 + 4K, LENGTH = 1M - 4K /* 4K for boot */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}
