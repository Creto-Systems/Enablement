import type {
  FlightBooking,
  HotelBooking,
  Activity,
  TripConstraints,
  FlightClass,
  ActivityType,
} from '@shared/types';
import { generateId } from './encryption.js';

/**
 * Mock data generators for demo purposes
 */

const AIRLINES = [
  'United Airlines',
  'Delta Air Lines',
  'American Airlines',
  'Southwest Airlines',
  'JetBlue Airways',
];

const AIRPORTS: Record<string, string> = {
  'New York': 'JFK',
  'Los Angeles': 'LAX',
  Paris: 'CDG',
  London: 'LHR',
  Tokyo: 'NRT',
  'San Francisco': 'SFO',
  Chicago: 'ORD',
  Miami: 'MIA',
};

/**
 * Generate mock flight options
 */
export function generateFlights(
  constraints: TripConstraints,
  agentId: string
): FlightBooking[] {
  const origin = 'SFO'; // Default origin
  const destination =
    AIRPORTS[constraints.destination] || constraints.destination.slice(0, 3).toUpperCase();

  const maxPrice = constraints.budget.max * 0.4;
  const flights: FlightBooking[] = [];

  // Generate 3-5 flight options
  for (let i = 0; i < 5; i++) {
    const airline = AIRLINES[i % AIRLINES.length];
    const stops = i < 2 ? 0 : i < 4 ? 1 : 2;
    const basePrice = 300 + stops * 150 + Math.random() * 200;
    const duration = 360 + stops * 120 + Math.random() * 60;

    const price =
      basePrice *
      (constraints.preferences.flightClass === 'business' ? 3 : 1) *
      constraints.travelerCount;

    if (price > maxPrice && i > 2) continue; // Skip expensive options after first few

    flights.push({
      id: generateId(),
      airline,
      flightNumber: `${airline.slice(0, 2).toUpperCase()}${Math.floor(Math.random() * 9000 + 1000)}`,
      departure: {
        airport: origin,
        time: new Date(constraints.startDate).toISOString(),
      },
      arrival: {
        airport: destination,
        time: new Date(
          new Date(constraints.startDate).getTime() + duration * 60000
        ).toISOString(),
      },
      duration,
      stops,
      price: Math.round(price),
      class: constraints.preferences.flightClass,
      agentId,
      status: 'suggested',
    });
  }

  return flights.sort((a, b) => {
    // Sort by composite score
    const scoreA = a.price * 0.4 + a.duration * 0.3 + a.stops * 100 * 0.3;
    const scoreB = b.price * 0.4 + b.duration * 0.3 + b.stops * 100 * 0.3;
    return scoreA - scoreB;
  });
}

/**
 * Generate mock hotel options
 */
export function generateHotels(
  constraints: TripConstraints,
  agentId: string
): HotelBooking[] {
  const nights = Math.ceil(
    (new Date(constraints.endDate).getTime() -
      new Date(constraints.startDate).getTime()) /
      (1000 * 60 * 60 * 24)
  );

  const maxPricePerNight = (constraints.budget.max * 0.35) / nights;
  const hotels: HotelBooking[] = [];

  const hotelNames = [
    'Grand Plaza Hotel',
    'City Center Inn',
    'Seaside Resort & Spa',
    'Downtown Boutique Hotel',
    'Airport Comfort Suites',
  ];

  for (let i = 0; i < 5; i++) {
    const starRating = 5 - i;
    const pricePerNight = 80 + starRating * 40 + Math.random() * 50;
    const totalPrice = pricePerNight * nights;

    if (totalPrice > maxPricePerNight * nights && i > 2) continue;

    hotels.push({
      id: generateId(),
      name: hotelNames[i],
      address: `${100 + i * 10} ${constraints.destination} Street`,
      starRating,
      checkIn: constraints.startDate,
      checkOut: constraints.endDate,
      roomType:
        starRating >= 4
          ? 'Deluxe Suite'
          : starRating >= 3
            ? 'Standard Room'
            : 'Economy Room',
      pricePerNight: Math.round(pricePerNight),
      totalPrice: Math.round(totalPrice),
      amenities:
        starRating >= 4
          ? ['Pool', 'Spa', 'Gym', 'Restaurant', 'Concierge']
          : starRating >= 3
            ? ['Wi-Fi', 'Breakfast', 'Gym']
            : ['Wi-Fi', 'Parking'],
      distanceToCenter: 0.5 + i * 0.8,
      agentId,
      status: 'suggested',
    });
  }

  return hotels;
}

/**
 * Generate mock activities
 */
export function generateActivities(
  constraints: TripConstraints,
  agentId: string
): Activity[] {
  const days = Math.ceil(
    (new Date(constraints.endDate).getTime() -
      new Date(constraints.startDate).getTime()) /
      (1000 * 60 * 60 * 24)
  );

  const maxPerDay =
    constraints.preferences.pace === 'packed'
      ? 4
      : constraints.preferences.pace === 'moderate'
        ? 3
        : 2;

  const activities: Activity[] = [];
  const activityTemplates: Record<
    ActivityType,
    { name: string; duration: number; price: number }[]
  > = {
    adventure: [
      { name: 'Zip Line Tour', duration: 180, price: 120 },
      { name: 'Rock Climbing', duration: 240, price: 95 },
      { name: 'Kayaking Adventure', duration: 150, price: 75 },
    ],
    culture: [
      { name: 'Museum Tour', duration: 120, price: 25 },
      { name: 'Historical Walking Tour', duration: 180, price: 35 },
      { name: 'Art Gallery Visit', duration: 90, price: 20 },
    ],
    relaxation: [
      { name: 'Spa Treatment', duration: 120, price: 150 },
      { name: 'Beach Day', duration: 240, price: 0 },
      { name: 'Yoga Class', duration: 60, price: 30 },
    ],
    food: [
      { name: 'Food Tour', duration: 180, price: 85 },
      { name: 'Cooking Class', duration: 150, price: 95 },
      { name: 'Wine Tasting', duration: 120, price: 65 },
    ],
    shopping: [
      { name: 'Local Markets Tour', duration: 120, price: 15 },
      { name: 'Shopping District Visit', duration: 180, price: 0 },
    ],
    nature: [
      { name: 'National Park Hike', duration: 300, price: 45 },
      { name: 'Wildlife Safari', duration: 240, price: 125 },
      { name: 'Botanical Gardens', duration: 90, price: 15 },
    ],
  };

  let currentDate = new Date(constraints.startDate);

  for (let day = 0; day < days; day++) {
    let activityCount = 0;
    let currentTime = 9; // Start at 9 AM

    for (const type of constraints.preferences.activityTypes) {
      if (activityCount >= maxPerDay) break;

      const templates = activityTemplates[type];
      const template = templates[Math.floor(Math.random() * templates.length)];

      activities.push({
        id: generateId(),
        name: template.name,
        type,
        description: `Experience the best ${type} activity in ${constraints.destination}`,
        date: new Date(currentDate).toISOString().split('T')[0],
        startTime: `${currentTime.toString().padStart(2, '0')}:00`,
        duration: template.duration,
        price: template.price * constraints.travelerCount,
        location: {
          address: `${constraints.destination} ${type} Center`,
          coordinates: {
            lat: 37.7749 + Math.random() * 0.1,
            lng: -122.4194 + Math.random() * 0.1,
          },
        },
        agentId,
        status: 'suggested',
      });

      currentTime += Math.ceil(template.duration / 60) + 1; // Add activity duration + 1 hour buffer
      activityCount++;
    }

    currentDate.setDate(currentDate.getDate() + 1);
  }

  return activities;
}

/**
 * Calculate total cost of items
 */
export function calculateTotalCost(
  flights: FlightBooking[],
  hotels: HotelBooking[],
  activities: Activity[]
): number {
  const flightCost = flights[0]?.price || 0;
  const hotelCost = hotels[0]?.totalPrice || 0;
  const activityCost = activities.reduce((sum, a) => sum + a.price, 0);

  return flightCost + hotelCost + activityCost;
}
