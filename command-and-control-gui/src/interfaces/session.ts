import {NetworkInterfaceArray} from "@/interfaces/agent";

export interface SessionRecord {
    id: string,
    hostname: string,
    domain: string,
    username: string,
    network_interfaces: NetworkInterfaceArray,
    integrity: string,
    operating_system: string,
}