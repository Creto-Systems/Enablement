export interface IAgentService {
  createAgent(dto: CreateAgentDTO): Promise<Agent>;
  getAgent(id: string): Promise<Agent | null>;
  terminateAgent(id: string): Promise<void>;
  listAgents(filters?: AgentFilters): Promise<Agent[]>;
  updateAgentStatus(id: string, status: AgentStatus): Promise<Agent>;
}

export interface CreateAgentDTO {
  name: string;
  type: string;
  config: Record<string, any>;
  userId: string;
}

export interface Agent {
  id: string;
  name: string;
  type: string;
  status: AgentStatus;
  config: Record<string, any>;
  userId: string;
  createdAt: Date;
  updatedAt?: Date;
}

export type AgentStatus = 'active' | 'inactive' | 'terminated' | 'error';

export interface AgentFilters {
  userId?: string;
  status?: AgentStatus;
  type?: string;
}
