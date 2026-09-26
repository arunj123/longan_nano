CXX ?= g++
CXXFLAGS ?= -std=c++23 -O2 -Wall -Wextra -pthread
TARGET = virtual_usb_msc

SRCS = main.cpp virtual_disk.cpp msc_scsi_engine.cpp usbip_server.cpp
OBJS = $(SRCS:.cpp=.o)

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CXX) $(CXXFLAGS) -o $@ $^

%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)
