# weather.py
"""
Handles fetching and parsing weather data from the Open-Meteo API.
"""
import requests

def get_weather(lat, lon):
    """Fetches current weather data."""
    url = f"https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current_weather=true"
    try:
        res = requests.get(url, timeout=10)
        res.raise_for_status()
        data = res.json()
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

def _map_weather_code_to_icon(code):
    """Maps WMO weather codes to our internal icon names."""
    if code in [0, 1]: return "sun"
    if code in [2]: return "sun_cloud"
    if code in [3, 45, 48]: return "cloud"
    if code in [51, 53, 55, 56, 57, 61, 63, 65, 66, 67, 80, 81, 82]: return "rain"
    if code in [71, 73, 75, 77, 85, 86]: return "snow"
    if code in [95, 96, 99]: return "storm"
    return "cloud"