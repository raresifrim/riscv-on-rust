use crate::risc_soc::memory_management_unit::MemoryDevice;
use crate::risc_soc::memory_management_unit::Address;
use crate::risc_soc::memory_management_unit::MemoryRequest;
use crate::risc_soc::memory_management_unit::MemoryRequestType;
use crate::risc_soc::memory_management_unit::MemoryResponse;
use crate::risc_soc::memory_management_unit::MemoryDeviceType;
use crate::risc_soc::memory_management_unit::MemoryResponseType;

pub struct UART {
    start_address: Address,
    end_address: Address,
}

impl MemoryDevice for UART {
    fn new(memory_type: MemoryDeviceType, start_address: Address, end_address: Address) -> Self {
        assert!(memory_type == MemoryDeviceType::UART0);
        Self { 
            start_address, 
            end_address
        }
    }

    fn send_data_request(&mut self, request: MemoryRequest) -> MemoryResponse {
        if request.request_type == MemoryRequestType::WRITE {
            assert!(request.data_address == self.start_address + 0x4 && request.data.is_some());
            for char in request.data.unwrap() {
                print!("{}", char as char);
            }
            MemoryResponse{
                data: vec![],
                status: MemoryResponseType::Valid
            }
        } else {
            panic!("The UART Device does not have the read operation implemented yet!")
        }   
    }

    fn read_request(&self, request: MemoryRequest) -> MemoryResponse {
        unimplemented!()
    }

    fn start_end_addresses(&self) -> (Address, Address) {
        (self.start_address, self.end_address)
    }

    fn get_memory_type(&self) -> MemoryDeviceType {
        MemoryDeviceType::UART0
    }

    fn init_mem(&mut self, address: Address, data: &[u8]) {
        unimplemented!()        
    }

    fn size(&self) -> usize {
        unimplemented!()
    }

    fn debug(&self, _start_address: Address, _end_address: Address) -> std::fmt::Result {
        unimplemented!()        
    }
}