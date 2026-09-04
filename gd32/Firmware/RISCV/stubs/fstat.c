/* See LICENSE of license details. */


#include <errno.h>
#include <unistd.h>
#include <sys/stat.h>
#include "stub.h"

__attribute__((used)) int _fstat(int fd, struct stat* st)
{
  if (_isatty(fd)) {
    st->st_mode = S_IFCHR;
    return 0;
  }

  return _stub(EBADF);
}
