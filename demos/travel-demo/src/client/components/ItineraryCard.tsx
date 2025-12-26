import React, { useState } from 'react';
import type { Itinerary } from '../../shared/types';
import { format } from 'date-fns';

interface ItineraryCardProps {
  itinerary: Itinerary;
}

const ItineraryCard: React.FC<ItineraryCardProps> = ({ itinerary }) => {
  const [expandedDay, setExpandedDay] = useState<number | null>(0);

  // Group items by day
  const getDaysData = () => {
    const days = new Map<string, any>();

    // Add flights
    itinerary.flights.forEach((flight) => {
      const date = format(new Date(flight.departure.time), 'yyyy-MM-dd');
      if (!days.has(date)) {
        days.set(date, { date, flights: [], hotels: [], activities: [] });
      }
      days.get(date)!.flights.push(flight);
    });

    // Add hotels
    itinerary.hotels.forEach((hotel) => {
      const date = hotel.checkIn;
      if (!days.has(date)) {
        days.set(date, { date, flights: [], hotels: [], activities: [] });
      }
      days.get(date)!.hotels.push(hotel);
    });

    // Add activities
    itinerary.activities.forEach((activity) => {
      const date = activity.date;
      if (!days.has(date)) {
        days.set(date, { date, flights: [], hotels: [], activities: [] });
      }
      days.get(date)!.activities.push(activity);
    });

    return Array.from(days.values()).sort((a, b) => a.date.localeCompare(b.date));
  };

  const daysData = getDaysData();

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'accepted':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'suggested':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'rejected':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
  };

  return (
    <div className="space-y-4">
      {/* Summary Card */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Total Cost</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              ${itinerary.totalCost.toLocaleString()}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Optimization Score</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {(itinerary.optimizationScore * 100).toFixed(0)}%
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Status</p>
            <p className={`inline-block px-3 py-1 rounded-full text-sm font-medium capitalize ${getStatusColor(itinerary.status)}`}>
              {itinerary.status}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500 dark:text-gray-400">Conflicts</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-white">
              {itinerary.conflicts.length}
            </p>
          </div>
        </div>
      </div>

      {/* Timeline */}
      <div className="space-y-3">
        {daysData.map((day, index) => (
          <div
            key={day.date}
            className="bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 overflow-hidden"
          >
            {/* Day Header */}
            <button
              onClick={() => setExpandedDay(expandedDay === index ? null : index)}
              className="w-full p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center">
                  <span className="text-xl font-bold text-primary">
                    {index + 1}
                  </span>
                </div>
                <div className="text-left">
                  <p className="font-semibold text-gray-900 dark:text-white">
                    {format(new Date(day.date), 'EEEE, MMMM d, yyyy')}
                  </p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {day.flights.length} flights, {day.hotels.length} hotels,{' '}
                    {day.activities.length} activities
                  </p>
                </div>
              </div>
              <svg
                className={`w-5 h-5 text-gray-400 transition-transform ${
                  expandedDay === index ? 'rotate-180' : ''
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </button>

            {/* Day Details */}
            {expandedDay === index && (
              <div className="p-4 border-t border-gray-200 dark:border-gray-700 space-y-4">
                {/* Flights */}
                {day.flights.map((flight) => (
                  <div
                    key={flight.id}
                    className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3">
                        <span className="text-2xl">✈️</span>
                        <div>
                          <p className="font-semibold text-gray-900 dark:text-white">
                            {flight.airline} {flight.flightNumber}
                          </p>
                          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {flight.departure.airport} → {flight.arrival.airport}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                            {format(new Date(flight.departure.time), 'h:mm a')} -{' '}
                            {format(new Date(flight.arrival.time), 'h:mm a')} ({flight.duration} min)
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-semibold text-gray-900 dark:text-white">
                          ${flight.price.toLocaleString()}
                        </p>
                        <span className={`inline-block px-2 py-1 rounded-full text-xs font-medium mt-1 ${getStatusColor(flight.status)}`}>
                          {flight.status}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}

                {/* Hotels */}
                {day.hotels.map((hotel) => (
                  <div
                    key={hotel.id}
                    className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg border border-purple-200 dark:border-purple-800"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3">
                        <span className="text-2xl">🏨</span>
                        <div>
                          <p className="font-semibold text-gray-900 dark:text-white">
                            {hotel.name}
                          </p>
                          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {'⭐'.repeat(hotel.starRating)} • {hotel.roomType}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                            {hotel.distanceToCenter.toFixed(1)} km to center
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-semibold text-gray-900 dark:text-white">
                          ${hotel.totalPrice.toLocaleString()}
                        </p>
                        <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                          ${hotel.pricePerNight}/night
                        </p>
                        <span className={`inline-block px-2 py-1 rounded-full text-xs font-medium mt-1 ${getStatusColor(hotel.status)}`}>
                          {hotel.status}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}

                {/* Activities */}
                {day.activities.map((activity) => (
                  <div
                    key={activity.id}
                    className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3">
                        <span className="text-2xl">🎭</span>
                        <div>
                          <p className="font-semibold text-gray-900 dark:text-white">
                            {activity.name}
                          </p>
                          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {activity.description}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                            {activity.startTime} • {activity.duration} min • {activity.type}
                          </p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-semibold text-gray-900 dark:text-white">
                          ${activity.price.toLocaleString()}
                        </p>
                        <span className={`inline-block px-2 py-1 rounded-full text-xs font-medium mt-1 ${getStatusColor(activity.status)}`}>
                          {activity.status}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Conflicts */}
      {itinerary.conflicts.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            ⚠️ Conflicts Detected
          </h3>
          <div className="space-y-3">
            {itinerary.conflicts.map((conflict) => (
              <div
                key={conflict.id}
                className={`p-4 rounded-lg border ${
                  conflict.severity === 'high'
                    ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                    : conflict.severity === 'medium'
                    ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800'
                    : 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800'
                }`}
              >
                <p className="font-medium text-gray-900 dark:text-white capitalize">
                  {conflict.type} Conflict
                </p>
                <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                  {conflict.description}
                </p>
                {conflict.resolution && (
                  <p className="text-xs text-green-600 dark:text-green-400 mt-2">
                    ✓ Resolved: {conflict.resolution.strategy}
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ItineraryCard;
