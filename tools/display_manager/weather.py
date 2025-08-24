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
        weather_info = {
            'temperature': int(round(current['temperature'])),
            'icon': _map_weather_code_to_icon(current['weathercode'])
        }
        
        print(f"Weather Updated: {weather_info['temperature']}°C, Condition: {weather_info['icon']}")
        return weather_info
        
    except requests.exceptions.RequestException as e:
        print(f"Error fetching weather: {e}")
        return None

def _map_weather_code_to_icon(code: int) -> str:
    """
    Maps WMO weather interpretation codes to our internal icon names.

    The codes are based on the WMO Code 4561 standard.
    Reference: https://open-meteo.com/en/docs#weathervariables

    Args:
        code (int): The integer weather code from the API.

    Returns:
        str: The corresponding icon name (e.g., "sun", "rain", "storm").
             Defaults to "cloud" for unknown codes.
    """
    # Clear sky or few clouds
    if code in [0, 1]: return "sun"
    # Scattered or broken clouds
    if code in [2]: return "sun_cloud"
    # Overcast, fog
    if code in [3, 45, 48]: return "cloud"
    # Drizzle, rain
    if code in [51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 80, 81, 82]: return "rain"
    # Snow, freezing rain
    if code in [71, 73, 75, 77, 85, 86]: return "snow"
    # Thunderstorm
    if code in [95, 96, 99]: return "storm"
    
    # Default to a generic cloud icon for any other codes
    return "cloud"