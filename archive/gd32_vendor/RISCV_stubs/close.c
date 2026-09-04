/* See LICENSE of license details. */


#include <errno.h>
#include "stub.h"

__attribute__((used)) int _close(int fd)
{
  return _stub(EBADF);
}
