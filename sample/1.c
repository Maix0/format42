/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   1.c                                                :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maix <marvin@42.fr>                        +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2023/08/28 13:53:54 by maix              #+#    #+#             */
/*   Updated: 2023/08/30 19:20:52 by maix             ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#ifndef FOO
# define FOO
# define BAR 5
# if BAR == 5
#  define FOOBAR
# endif
#endif
// ba

#include "locale.h"
#include <stdio.h>

void exit(int);

// outer comment
// This is also a two line comment !
int main(int argc, char **argv) {
  // inner comment
  if (0)
    (void)5;
  // return 0;
}

/*
multiline comment
fhdjskfjsdhfkjsdhqjfkdhskfhdjqshfkjdshqkjfdhsqjfhdjkqsfjsqjkhfsdjqhfdsq
*/
