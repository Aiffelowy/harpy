def fibonacci(a):
    if a <= 1:
        return a

    return fibonacci(a-1) + fibonacci(a-2)

def main():
    n = 5
    result = fibonacci(n)
    print(result)

main()
