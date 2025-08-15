/*!
    \file    flash_operation.c
    \brief   flash operation driver

    \version 2025-02-10, V1.5.0, firmware for GD32VF103
*/

/*
    Copyright (c) 2025, GigaDevice Semiconductor Inc.

    Redistribution and use in source and binary forms, with or without modification, 
are permitted provided that the following conditions are met:

    1. Redistributions of source code must retain the above copyright notice, this 
       list of conditions and the following disclaimer.
    2. Redistributions in binary form must reproduce the above copyright notice, 
       this list of conditions and the following disclaimer in the documentation 
       and/or other materials provided with the distribution.
    3. Neither the name of the copyright holder nor the names of its contributors 
       may be used to endorse or promote products derived from this software without 
       specific prior written permission.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" 
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED 
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. 
IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, 
INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT 
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR 
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, 
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) 
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY 
OF SUCH DAMAGE.
*/

#include "flash_operation.h"
#include "riscv_encoding.h"

/*!
    \brief      erase flash
    \param[in]  address: erase start address
    \param[in]  file_length: file length
    \param[out] none
    \retval     state of FMC, refer to fmc_state_enum
*/
fmc_state_enum flash_erase(uint32_t address, uint32_t file_length)
{
    uint16_t i = 0U;
    fmc_state_enum fmc_state;
    uint16_t page_count;

    if(0U == (file_length % PAGE_SIZE)){
        page_count = (uint16_t)(file_length / PAGE_SIZE);
    }else{
        page_count = (uint16_t)(file_length / PAGE_SIZE + 1U);
    }

    /* clear pending flags */
    fmc_flag_clear(FMC_FLAG_PGERR | FMC_FLAG_WPERR | FMC_FLAG_END);

    for(i = 0U; i < page_count; i ++) {
        /* call the standard flash erase-page function */
        fmc_state = fmc_page_erase(address);

        address += PAGE_SIZE;
    }
    return fmc_state;
}


/*!
    \brief      write data to sectors of memory
    \param[in]  data: data to be written
    \param[in]  addr: sector address/code
    \param[in]  len: length of data to be written (in bytes)
    \param[out] none
    \retval     state of FMC, refer to fmc_state_enum
*/
fmc_state_enum iap_data_write(uint8_t *data, uint32_t addr, uint32_t len)
{
    uint32_t idx = 0U;
    fmc_state_enum status = FMC_READY ;

    /* check if the address is in protected area */
    if(IS_PROTECTED_AREA(addr)) {
        return FMC_BUSY;
    }

    /* unlock the flash program erase controller */
    fmc_unlock();

    /* clear pending flags */
    fmc_flag_clear(FMC_FLAG_PGERR | FMC_FLAG_WPERR | FMC_FLAG_END);

    /* data received are word multiple */
    for(idx = 0U; idx < len; idx += 2) {
        status = fmc_halfword_program(addr, *(uint32_t *)(data + idx));
        if(FMC_READY == status){
            addr += 2U;
        }else{
            while(1){
            }
        }
    }

    fmc_lock();

    return status;
}

/*!
    \brief      program option byte
    \param[in]  Mem_Add: target address
    \param[in]  data: pointer to target data
    \param[in]  len: length of data to be written (in bytes)
    \param[out] none
    \retval     state of FMC, refer to fmc_state_enum
*/
fmc_state_enum option_byte_write(uint32_t Mem_Add,uint8_t* data, uint16_t len)
{
    fmc_state_enum status ;

    /* unlock the flash program erase controller */
    fmc_unlock();

    /* clear pending flags */
    fmc_flag_clear(FMC_FLAG_PGERR | FMC_FLAG_WPERR  | FMC_FLAG_END);

    status = fmc_ready_wait(FMC_TIMEOUT_COUNT);

    /* authorize the small information block programming */
    ob_unlock();

    /* start erase the option byte */
    FMC_CTL |= FMC_CTL_OBER;
    FMC_CTL |= FMC_CTL_START;
    
    status = fmc_ready_wait(FMC_TIMEOUT_COUNT);

    FMC_CTL &= ~FMC_CTL_OBER;
    /* set the OBPG bit */
    FMC_CTL |= FMC_CTL_OBPG;

    uint8_t index;

    /*option bytes always have 16Bytes*/
    for(index = 0U; index < len; index = index+2U){
        *(__IO uint16_t*)Mem_Add = data[index]&0xffU;
        Mem_Add = Mem_Add + 2U;
        status = fmc_ready_wait(FMC_TIMEOUT_COUNT);
    }

    /* if the program operation is completed, disable the OBPG Bit */
    FMC_CTL &= ~FMC_CTL_OBPG;

    fmc_lock();

    return status;
}

/*!
    \brief      jump to execute address
    \param[in]  addr: execute address
    \param[out] none
    \retval     none
*/
void jump_to_execute(uint32_t addr)
{
    if (0x0001AAB1 == (*(__IO uint32_t*)addr)) {
        clear_csr(mstatus, MSTATUS_MIE);
        asm volatile ("jr %0 " :: "r"(addr));
    }
}