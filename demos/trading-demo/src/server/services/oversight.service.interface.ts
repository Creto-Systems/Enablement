export interface IOversightService {
  getPendingRequests(filters: OversightFilters): Promise<OversightRequest[]>;
  getRequest(id: string): Promise<OversightRequest | null>;
  approveRequest(id: string, approvedBy: string): Promise<OversightRequest>;
  rejectRequest(id: string, rejectedBy: string, reason: string): Promise<OversightRequest>;
  createRequest(dto: CreateOversightRequestDTO): Promise<OversightRequest>;
}

export interface OversightRequest {
  id: string;
  agentId: string;
  tradeId: string;
  reason: string;
  status: OversightStatus;
  createdAt: Date;
  approvedBy?: string;
  approvedAt?: Date;
  rejectedBy?: string;
  rejectedAt?: Date;
  rejectionReason?: string;
}

export type OversightStatus = 'pending' | 'approved' | 'rejected' | 'expired';

export interface OversightFilters {
  agentId?: string;
  status?: OversightStatus;
}

export interface CreateOversightRequestDTO {
  agentId: string;
  tradeId: string;
  reason: string;
}
