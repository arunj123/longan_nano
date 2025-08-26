# weather.py
"""
Handles fetching and parsing weather data from the free Open-Meteo API.

This module provides a simple interface to get current weather conditions
for a given latitude and longitude.
"""
import requests
from typing import Optional, Dict, Any

def get_weather(lat: float, lon: float) -> Optional[Dict[str, Any]]:
    """
    Fetches current weather data from the Open-Meteo API.

    Args:
        lat (float): The latitude for the weather forecast.
        lon (float): The longitude for the weather forecast.

    Returns:
        A dictionary containing 'temperature' and 'icon' on success,
        or None if an error occurs (e.g., network issue, API error).
        Example: {'temperature': 15, 'icon': 'sun_cloud'}
    """
    url = f"https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true"
    try:
        response = requests.get(url, timeout=10)
        response.raise_for_status()  # Raises an HTTPError for bad responses (4xx or 5xx)
        data = response.json()
        
        current = data['current_weather']
        # is_day is 1 for daytime, 0 for nighttime. Default to 1 (day) if not present.
        is_day = current.get('is_day', 1)
        weather_info = {
            'temperature': int(round(current['temperature'])),
            'icon': _map_weather_code_to_icon(current['weathercode'], is_day),
            'is_day': is_day
        }
        
        print(f"Weather Updated: {weather_info['temperature']}°C, Condition: {weather_info['icon']} (Day: {weather_info['is_day']})")
        return weather_info
        
    except requests.exceptions.RequestException as e:
        print(f"Error fetching weather: {e}")
        return None

def _map_weather_code_to_icon(code: int, is_day: int) -> str:
    """
    Maps WMO weather interpretation codes to our internal icon names,
    accounting for day and night.

    The codes are based on the WMO Code 4561 standard.
    Reference: https://open-meteo.com/en/docs#weathervariables

    Args:
        code (int): The integer weather code from the API.
        is_day (int): 1 if it is daytime, 0 if it is nighttime.

    Returns:
        str: The corresponding icon name (e.g., "sun", "rain", "storm").
             Defaults to "cloud" for unknown codes.
    """
    # Clear sky or few clouds
    if code in [0, 1]:
        return "sun" if is_day else "moon"
    # Scattered or broken clouds
    if code in [2]:
        return "sun_cloud" if is_day else "moon_cloud"
    # Overcast
    if code in [3]: return "cloud"
    # Fog
    if code in [45, 48]: return "fog"
    # Drizzle
    if code in [51, 53, 55]: return "drizzle"
    # Freezing Drizzle/Rain
    if code in [56, 57, 66, 67]: return "freezing_rain"
    # Light Rain
    if code in [61, 80]: return "light_rain"
    # Moderate Rain
    if code in [63, 81]: return "rain"
    # Heavy Rain
    if code in [65, 82]: return "heavy_rain"
    # Light Snow
    if code in [71, 77, 85]: return "light_snow"
    # Moderate Snow
    if code in [73]: return "snow"
    # Heavy Snow
    if code in [75, 86]: return "heavy_snow"
    # Thunderstorm
    if code in [95]: return "storm"
    # Thunderstorm with Hail
    if code in [96, 99]: return "storm_hail"
    # Default to a generic cloud icon for any other codes
    return "cloud"