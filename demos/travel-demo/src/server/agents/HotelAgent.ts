import { BaseAgent } from './BaseAgent.js';
import type {
  AgentMessage,
  TripConstraints,
  HotelBooking,
} from '@shared/types';
import { generateHotels } from '../utils/mockData.js';

/**
 * Hotel Agent - Searches and ranks hotel options
 */
export class HotelAgent extends BaseAgent {
  constructor(tripId: string) {
    super('hotel', tripId);
  }

  protected async processMessage(message: AgentMessage): Promise<void> {
    const { action, data } = message.payload;

    switch (action) {
      case 'startPlanning':
        await this.handleStartPlanning(data as { constraints: TripConstraints });
        break;

      case 'requestAlternative':
        await this.handleAlternativeRequest(message.from);
        break;

      default:
        console.log(`HotelAgent: Unknown action ${action}`);
    }
  }

  private async handleStartPlanning(data: {
    constraints: TripConstraints;
  }): Promise<void> {
    this.updateStatus('working');

    // Simulate search delay
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const results = await this.search(data.constraints);

    this.updateStatus('completed');
  }

  private async handleAlternativeRequest(requesterId: string): Promise<void> {
    this.updateStatus('working');

    // Generate alternative options
    await new Promise((resolve) => setTimeout(resolve, 600));

    this.updateStatus('completed');
  }

  async search(constraints: TripConstraints): Promise<HotelBooking[]> {
    // Generate mock hotel data
    const hotels = generateHotels(constraints, this.id);

    // Rank and select top options
    const rankedHotels = this.rankHotels(hotels, constraints);
    const topHotels = rankedHotels.slice(0, 5);

    // Send results to coordinator
    await this.sendResults('coordinator', {
      hotels: topHotels,
      category: 'hotels',
    });

    // Notify budget agent
    if (topHotels.length > 0) {
      await this.notifyBudget('hotels', topHotels[0].totalPrice);
    }

    return topHotels;
  }

  /**
   * Rank hotels by composite score
   */
  private rankHotels(
    hotels: HotelBooking[],
    constraints: TripConstraints
  ): HotelBooking[] {
    return hotels.sort((a, b) => {
      const scoreA = this.calculateHotelScore(a, constraints);
      const scoreB = this.calculateHotelScore(b, constraints);
      return scoreB - scoreA;
    });
  }

  /**
   * Calculate hotel score
   */
  private calculateHotelScore(
    hotel: HotelBooking,
    constraints: TripConstraints
  ): number {
    const nights = Math.ceil(
      (new Date(constraints.endDate).getTime() -
        new Date(constraints.startDate).getTime()) /
        (1000 * 60 * 60 * 24)
    );

    const maxPricePerNight = (constraints.budget.max * 0.35) / nights;

    // Star rating score
    const ratingScore = hotel.starRating / 5.0;

    // Price score (lower is better within budget)
    const priceScore = 1 - hotel.pricePerNight / maxPricePerNight;

    // Location score (closer to center is better)
    const locationScore = 1 - hotel.distanceToCenter / 10; // 10km max

    // Amenity score
    const amenityScore = Math.min(hotel.amenities.length / 5, 1);

    // Weighted combination
    return (
      ratingScore * 0.25 +
      priceScore * 0.35 +
      locationScore * 0.25 +
      amenityScore * 0.15
    );
  }
}
