/* See LICENSE of license details. */


#include <stdint.h>
#include <errno.h>
#include <unistd.h>
#include <sys/types.h>

#include "stub.h"

__attribute__((used)) ssize_t _read(int fd, void* ptr, size_t len)
{
  (void)fd;
  (void)ptr;
  (void)len;
  return _stub(EBADF);
}
