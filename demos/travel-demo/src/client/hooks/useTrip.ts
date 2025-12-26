import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import type { Trip, CreateTripRequest, Itinerary } from '../../shared/types';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const useTrip = () => {
  const [tripId, setTripId] = useState<string | null>(null);

  // Fetch trip data
  const { data: trip, isLoading: tripLoading } = useQuery<Trip>({
    queryKey: ['trip', tripId],
    queryFn: async () => {
      if (!tripId) throw new Error('No trip ID');
      const response = await fetch(`${API_BASE}/api/trips/${tripId}`);
      if (!response.ok) throw new Error('Failed to fetch trip');
      return response.json();
    },
    enabled: !!tripId,
  });

  // Fetch itinerary
  const { data: itinerary, isLoading: itineraryLoading } = useQuery<Itinerary>({
    queryKey: ['itinerary', tripId],
    queryFn: async () => {
      if (!tripId) throw new Error('No trip ID');
      const response = await fetch(`${API_BASE}/api/trips/${tripId}/itinerary`);
      if (!response.ok) throw new Error('Failed to fetch itinerary');
      return response.json();
    },
    enabled: !!tripId,
    refetchInterval: 2000, // Poll every 2 seconds for updates
  });

  // Create trip mutation
  const createTripMutation = useMutation({
    mutationFn: async (request: CreateTripRequest) => {
      const response = await fetch(`${API_BASE}/api/trips`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Failed to create trip');
      }

      return response.json();
    },
    onSuccess: (data) => {
      setTripId(data.tripId);
    },
  });

  return {
    trip,
    itinerary,
    loading: tripLoading || itineraryLoading || createTripMutation.isPending,
    createTrip: createTripMutation.mutateAsync,
  };
};
