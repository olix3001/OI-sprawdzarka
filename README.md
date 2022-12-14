# Sprawdzarka do OI

Ten program automatyzuje sprawdzanie programu na dużej ilości testów. <br/>
Stworzyłem go aby automatycznie testować swoje programy na OI.

## Instalacja
Najłatwiejszym sposobem na instalacje jest wpisanie komendy `cargo install --git https://github.com/olix3001/OI-sprawdzarka`. Następnie dostępne będzie w systemie polecenie `sprawdzarka-oi`.

Innym sposobem jest ręczne skompilowanie źródła lub użycie plików z zakładki releases na github.

## Screenshot

![Sprawdzarka](./images/sprawdzarka.jpg)

## Sposób użycia

Aby dowiedzieć się jak korzystać z tego programu wywołaj go z flagą `--help`. <br/>
Program przyjmuje pliki wykonywalne, nie należy podawać plików z kodem źródłowym. <br/>
`input` i `output` powinny być podane jako foldery, a pliki powinny być ponazywane odpowiednio `<nazwa>.in` dla wejścia, oraz `<nazwa>.out` dla wyjścia

## Zastrzeżenia

Ze względu na konstrukcję w niektórych przypadkach program może pokazywać wolniejszy czas wykonania niż jest normalnie.

Wersja na windows jest niestabilna i wolniejsza.

## Python

Python nie jest wspierany.

### **Nie gwarantuję, że program będzie pasował do twoich potrzeb, ani, że będzie utrzymywany!**
