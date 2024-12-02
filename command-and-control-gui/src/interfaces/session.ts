import {NetworkInterfaceArray} from "@/interfaces/agent";

export interface SessionRecord {
    id: string,
    hostname: string,
    domain: string,
    username: string,
    network_interfaces: NetworkInterfaceArray,
    integrity_level: number,
    operating_system: string,
}