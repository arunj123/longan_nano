/* See LICENSE of license details. */


#ifndef _NUCLEI_SYS_STUB_H
#define _NUCLEI_SYS_STUB_H

#include <stdint.h>
#include <stddef.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>

#ifdef __cplusplus
extern "C" {
#endif

void write_hex(int fd, unsigned long int hex);
__attribute__((used)) int _isatty(int fd);
__attribute__((used)) int _fstat(int fd, struct stat *st);
__attribute__((used)) off_t _lseek(int fd, off_t ptr, int dir);
__attribute__((used)) int _close(int fd);
__attribute__((used)) ssize_t _read(int fd, void *ptr, size_t len);
__attribute__((used)) void _exit(int code);
__attribute__((used)) void *_sbrk(ptrdiff_t incr);

static inline int _stub(int err)
{
  (void)err;
  return -1;
}

#ifdef __cplusplus
}
#endif

#endif /* _NUCLEI_SYS_STUB_H */

