#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/wait.h>

unsigned long start_address;

int read_process_map() {
    FILE *fp;
    char path[256];
    char buffer[1024];
    char start_address_str[12]; // 16文字のアドレスと終端文字用

    // 現在のプロセスのマップファイルパス
    snprintf(path, sizeof(path), "/proc/self/maps");

    // ファイルを開く
    fp = fopen(path, "r");
    if (fp == NULL) {
        perror("Failed to open maps file");
        return -1;
    }

    int read = 0;
    // ファイルの内容を読み取り表示
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        if (read == 0) {
            sscanf(buffer, "%12s", start_address_str);
            start_address = strtoul(start_address_str, NULL, 16);
            printf("The start address of the first memory range is: %lx\n", start_address);
            read = 1;
        }

        printf("%s", buffer);
    }

    // ファイルを閉じる
    fclose(fp);

    return 0;
}

int main() {
    int pid = getpid();
    printf("PID is %d\n", pid);

    read_process_map();

    printf("main:          %p\n", main);
    printf("start_address: %lx\n", start_address);
    printf("relative main: %p\n", main - start_address);

    return 0;
}
