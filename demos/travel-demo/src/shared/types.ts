// Shared types used across client and server

export type AgentType = 'flight' | 'hotel' | 'activity' | 'budget';
export type AgentStatus = 'idle' | 'working' | 'completed' | 'error';
export type MessageType = 'request' | 'response' | 'notification' | 'error';
export type MessagePriority = 'critical' | 'high' | 'medium' | 'low';
export type TripStatus = 'draft' | 'planning' | 'completed' | 'booked';
export type ItemStatus = 'suggested' | 'accepted' | 'rejected';
export type ConflictType = 'time' | 'budget' | 'location';
export type ConflictSeverity = 'low' | 'medium' | 'high';
export type FlightClass = 'economy' | 'premium' | 'business' | 'first';
export type AccommodationType = 'hotel' | 'hostel' | 'airbnb' | 'resort';
export type Pace = 'relaxed' | 'moderate' | 'packed';
export type ActivityType =
  | 'adventure'
  | 'culture'
  | 'relaxation'
  | 'food'
  | 'shopping'
  | 'nature';

// Trip Models
export interface Trip {
  id: string;
  userId: string;
  destination: string;
  startDate: string; // ISO date string
  endDate: string; // ISO date string
  budget: Budget;
  travelerCount: number;
  preferences: Preferences;
  status: TripStatus;
  createdAt: string;
  updatedAt: string;
}

export interface Budget {
  min: number;
  max: number;
  currency: string;
}

export interface Preferences {
  activityTypes: ActivityType[];
  accommodationType: AccommodationType;
  flightClass: FlightClass;
  pace: Pace;
}

// Itinerary Models
export interface Itinerary {
  id: string;
  tripId: string;
  flights: FlightBooking[];
  hotels: HotelBooking[];
  activities: Activity[];
  totalCost: number;
  optimizationScore: number;
  conflicts: Conflict[];
  status: 'building' | 'ready' | 'confirmed';
}

export interface FlightBooking {
  id: string;
  airline: string;
  flightNumber: string;
  departure: FlightLocation;
  arrival: FlightLocation;
  duration: number; // minutes
  stops: number;
  price: number;
  class: FlightClass;
  agentId: string;
  status: ItemStatus;
}

export interface FlightLocation {
  airport: string;
  time: string; // ISO datetime string
}

export interface HotelBooking {
  id: string;
  name: string;
  address: string;
  starRating: number;
  checkIn: string; // ISO date string
  checkOut: string; // ISO date string
  roomType: string;
  pricePerNight: number;
  totalPrice: number;
  amenities: string[];
  distanceToCenter: number; // km
  agentId: string;
  status: ItemStatus;
}

export interface Activity {
  id: string;
  name: string;
  type: ActivityType;
  description: string;
  date: string; // ISO date string
  startTime: string; // HH:MM format
  duration: number; // minutes
  price: number;
  location: Location;
  agentId: string;
  status: ItemStatus;
}

export interface Location {
  address: string;
  coordinates: Coordinates;
}

export interface Coordinates {
  lat: number;
  lng: number;
}

// Agent Models
export interface Agent {
  id: string;
  type: AgentType;
  status: AgentStatus;
  tripId: string;
  publicKey: string;
  lastActive: string;
  metrics: AgentMetrics;
}

export interface AgentMetrics {
  responseTime: number;
  successRate: number;
  messagesProcessed: number;
}

// Message Models
export interface AgentMessage {
  id: string;
  from: string;
  to: string;
  type: MessageType;
  payload: MessagePayload;
  encrypted: boolean;
  signature?: string;
  timestamp: number;
  correlationId: string;
}

export interface MessagePayload {
  action: string;
  data?: unknown;
  constraints?: TripConstraints;
}

export interface EncryptedMessage {
  envelope: MessageEnvelope;
  payload: string;
  signature: string;
  nonce: string;
}

export interface MessageEnvelope {
  from: string;
  to: string;
  timestamp: number;
  correlationId: string;
}

// Constraint Models
export interface TripConstraints {
  destination: string;
  startDate: string;
  endDate: string;
  budget: Budget;
  travelerCount: number;
  preferences: Preferences;
  userLocation?: string;
}

// Conflict Models
export interface Conflict {
  id: string;
  type: ConflictType;
  severity: ConflictSeverity;
  description: string;
  affectedItems: string[];
  resolution?: ConflictResolution;
}

export interface ConflictResolution {
  strategy: string;
  appliedBy: string;
  timestamp: string;
}

// Budget Models
export interface BudgetAnalysis {
  totalCost: number;
  totalWithBuffer: number;
  budget: Budget;
  breakdown: CostBreakdown;
  overBudget: boolean;
  underBudget: boolean;
  utilizationRate: number;
  suggestions: OptimizationSuggestion[];
}

export interface CostBreakdown {
  flights: number;
  hotels: number;
  activities: number;
  buffer: number;
}

export interface OptimizationSuggestion {
  category: 'flights' | 'hotels' | 'activities';
  action: string;
  potentialSavings?: number;
  additionalCost?: number;
}

export interface BudgetAlert {
  severity: 'low' | 'medium' | 'high';
  message: string;
  currentTotal: number;
  budgetMax: number;
}

// WebSocket Event Types
export interface ItineraryUpdate {
  type: 'flight' | 'hotel' | 'activity' | 'budget';
  item: FlightBooking | HotelBooking | Activity | BudgetAnalysis;
  action: 'added' | 'updated' | 'removed';
}

export interface ErrorEvent {
  code: string;
  message: string;
  details?: unknown;
}

// API Request/Response Types
export interface CreateTripRequest {
  destination: string;
  startDate: string;
  endDate: string;
  budget: Budget;
  travelerCount: number;
  preferences: Preferences;
}

export interface CreateTripResponse {
  tripId: string;
}

export interface UpdateItemRequest {
  status: ItemStatus;
}

// State Types
export interface TripState {
  tripId: string;
  data: Record<string, unknown>;
  version: number;
  lastUpdated: number;
}

// Utility Types
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};
